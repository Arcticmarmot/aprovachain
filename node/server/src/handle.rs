use tx::envelope::{TxEnvelopeWire, TxEnvelope};
use axum::body::{Bytes};
use axum::extract::State;
use axum::Json;
use risc0_zkvm::{default_prover, Digest, ExecutorEnv, ProverOpts, Prover};
use tx::intent::{TxIntent, TxPayload};
use crate::error::{ApiResult, ServerError, Result};
use primitives::hash::{sha256, Hash32};
use account::address::{ContractAddress};
use account::keypair::AccountVerifyingKey;
use contract::contract::{Contract};
use db::handle::DBHandle;
use apps::ctr_io::{AccessSet, CtrContext, CtrInput, CtrOutput, CtrResult, ReadSet};
use tx::outcome::{TxOutcome};
use tx::attestation::TxAttestation;
use crate::context::{AppState, SubmitTxResponse};
use crate::error::ServerError::ProofGenerate;

/// 交易提交处理函数
pub async fn submit_tx(State(state) : State<AppState>, tx_bytes: Bytes) -> ApiResult<SubmitTxResponse> {
    let db_handle = state.db_handle;
    let cmd_handle = state.cmd_handle;
    let sk = state.sk;
    // 从字节数组构造 TxEnvelope
    let tx_envelope_wire: TxEnvelopeWire = TxEnvelopeWire::try_decode_bcs(tx_bytes.as_ref())?;
    let tx_envelope = TxEnvelope::try_from(tx_envelope_wire)?;
    tracing::info!(target:"node::server", tx_envelope_id=%tx_envelope.tx_id(), "tx envelope id");
    
    // TODO: 重放交易攻击，拒绝重复的 nonce
    // 验证交易签名是否有效
    verify_tx_sig(&tx_envelope)?;
    
    // 执行交易
    let response = handle_intent(db_handle, &tx_envelope.verifying_key, &tx_envelope.intent)?;
    match &response {
        SubmitTxResponse::Deploy { .. } => {
            let tx_outcome = TxOutcome {
                envelope: tx_envelope,
                receipt_opt: None
            };
            tracing::info!(target: "node::server", len=?tx_outcome.envelope.to_canonical_bytes().len(), "tx envelope size");
            tracing::info!(target: "node::server", len=?tx_outcome.to_canonical_bytes().len(), "tx outcome size");
            let tx = TxAttestation::create(tx_outcome, sk);
            cmd_handle.publish_tx(tx.to_canonical_bytes())?;
        },
        SubmitTxResponse::Exec {receipt, ..} => {
            let tx_outcome = TxOutcome {
                envelope: tx_envelope,
                receipt_opt: Some(receipt.clone()),
            };
            tracing::info!(target: "node::server", len=?tx_outcome.envelope.to_canonical_bytes().len(), "tx envelope size");
            tracing::info!(target: "node::server", len=?tx_outcome.to_canonical_bytes().len(), "tx outcome size");
            let tx = TxAttestation::create(tx_outcome, sk);
            cmd_handle.publish_tx(tx.to_canonical_bytes())?;
        }
    };
    Ok(Json(response))
}

pub fn handle_intent(db_handle: DBHandle, vk: &AccountVerifyingKey, intent: &TxIntent) -> Result<SubmitTxResponse> {
    let payload = &intent.payload;
    match payload {
        TxPayload::Deploy{ image_id, elf, elf_hash } => {
            handle_deploy_tx(db_handle, intent, vk, image_id, elf, elf_hash)
        },
        TxPayload::Exec { ctr_addr_str, input, access_set} => {
            handle_exec_tx(db_handle, intent, ctr_addr_str, input, access_set)
        }
    }
}

/// 验证交易签名
pub fn verify_tx_sig(envelope: &TxEnvelope) -> Result<()> {
    let vk = &envelope.verifying_key;
    let intent_id_hash = envelope.intent.tx_id().0;
    vk.verify(&intent_id_hash, &envelope.signature)?;
    Ok(())
}

pub fn handle_deploy_tx(_: DBHandle, intent: &TxIntent, vk: &AccountVerifyingKey, 
                        image_id: &Digest, elf: &Vec<u8>, elf_hash: &Hash32) -> Result<SubmitTxResponse> {
    // 验证 ELF 文件哈希是否对应
    let computed_elf_hash = sha256(elf);
    if &computed_elf_hash != elf_hash {
        return Err(ServerError::ElfHashMismatch)
    }
    // 验证 image_id 是否对应
    let computed_image_id = risc0_zkvm::compute_image_id(elf).map_err(ServerError::ImageIdCompute)?;
    if &computed_image_id != image_id {
        return Err(ServerError::ImageIdMismatch)
    }
    tracing::info!(target: "node::server", image_id=?image_id.as_words());
    tracing::info!(target: "node::server", elf_hash=?elf_hash);

    // 模拟创建合约
    let ctr = Contract::create(intent.chain_id, elf_hash, image_id, vk, intent.nonce);
    // 获取合约Bech32m编码
    let ctr_addr_str =  ctr.addr.to_bech32m()?;
    tracing::info!("{}", ctr_addr_str);

    Ok(SubmitTxResponse::Deploy {
        ctr_addr_str,
        image_id: computed_image_id,
        elf_hash: computed_elf_hash
    })
}

pub fn handle_exec_tx(db_handle: DBHandle, intent: &TxIntent, ctr_addr_str: &String,
                      input: &Vec<u8>, access_set: &AccessSet) -> Result<SubmitTxResponse> {
    tracing::info!(target: "node::handle", tx_id=?intent.tx_id());
    // 根据合约地址查找合约 BCS 编码向量
    let ctr_addr = ContractAddress::parse_bech32m_with_id(intent.chain_id, ctr_addr_str)?;
    let ctr_addr_bytes = ctr_addr.to_bytes();
    let ctr = match db_handle.load_contract(&ctr_addr_bytes)? {
        Some(ctr) => ctr,
        None => return {
            tracing::warn!(target: "node::server", "Contract not found");
            Err(ServerError::ContractNotFound)
        }
    };

    tracing::info!(target: "node::server", "Contract: {:?}", ctr);

    let image_id = ctr.image_id;
    // 从合约中获取 elf_hash
    let elf_hash = ctr.elf_hash;

    // 根据 ELF 文件哈希查找 ELF 文件
    let elf = match db_handle.load_elf(elf_hash)? {
        Some(elf) => elf,
        None => return {
            tracing::warn!(target: "node::server", "Elf file not found");
            Err(ServerError::ElfFileNotFound)
        }
    };
    tracing::info!("Elf file len: {}", elf.len());

    // 加载 access_set 数据
    let mut read_set: ReadSet = ReadSet::new();
    for ns_key in access_set {
        let value = db_handle.load_data_entry(ns_key)?;
        read_set.push((ns_key.clone(), value));
    }
    tracing::info!(target: "node::server", read_set=?read_set);
    let ctr_ctx = CtrContext {
        chain_id: intent.chain_id,
        input: input.clone(),
        read_set,
    };
    let ctr_input = CtrInput::create(&ctr_ctx);

    // 搭建虚拟机环境传入 input
    let env = ExecutorEnv::builder()
        .write(&ctr_input.encode_bcs())
        .unwrap()
        .build().map_err(ServerError::ExecutorEnvBuild)?;

    // opts 里选 succinct
    let opt = ProverOpts::groth16();

    // 根据虚拟机环境和 ELF 文件生成证明
    let prover = default_prover();

    let proof = prover.prove_with_opts(env, &elf, &opt).map_err(ProofGenerate)?;

    tracing::info!(target: "node::server", ?proof);
    let receipt = proof.receipt;
    let ctr_output_bytes: Vec<u8> = receipt.journal.decode()?;
    let ctr_output = CtrOutput::try_decode_bcs(&ctr_output_bytes)?;
    tracing::info!(target: "node::server", ?ctr_output);
    match ctr_output.ctr_result {
        CtrResult::Ok { outcome } => {
            Ok(SubmitTxResponse::Exec {
                ctr_addr_str: ctr_addr_str.clone(),
                image_id,
                elf_hash,
                input: input.clone(),
                receipt,
                answer: outcome.answer
            })
        },
        CtrResult::Err { message } => {
            Err(ServerError::ContractExec { message })
        }
    }
}
