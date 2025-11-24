use tx::envelope::{TxEnvelopeWire, TxEnvelope};
use axum::body::{Bytes};
use axum::extract::State;
use axum::Json;
use risc0_zkvm::{default_prover, Digest, ExecutorEnv, Prover};
use tx::intent::{TxIntent, TxPayload};
use crate::error::{ApiResult, ServerError, Result};
use primitives::hash::{sha256, Hash32};
use account::address::ChainAddrBytes;
use contract::contract::{Contract};
use db::handle::DBHandle;
use tx::outcome::{TxOutcome};
use tx::attestation::TxAttestation;
use crate::context::{AppState, SubmitTxResponse};

/// 交易提交处理函数
pub async fn submit_tx(State(state) : State<AppState>, tx_bytes: Bytes) -> ApiResult<SubmitTxResponse> {
    let db_handle = state.db_handle;
    let cmd_handle = state.cmd_handle;
    let sk = state.sk;
    // 从字节数组构造 TxEnvelope
    let tx_envelope_wire: TxEnvelopeWire = TxEnvelopeWire::try_decode_bcs(tx_bytes.as_ref())?;
    let tx_envelope = TxEnvelope::try_from(tx_envelope_wire)?;
    tracing::info!(target:"node::server", tx_envelope_id=%tx_envelope.tx_id(), "tx envelope id");
    // 验证交易签名是否有效
    // TODO: 重放交易攻击，拒绝重复的 nonce
    verify_tx_sig(&tx_envelope)?;
    
    // 执行交易
    let intent = handle_intent(db_handle, &tx_envelope.intent)?;
    match intent.clone() {
        SubmitTxResponse::Deploy{ .. } => {
            let tx_outcome = TxOutcome {
                envelope: tx_envelope,
                receipt_opt: None
            };
            tracing::info!(target: "node::server", len=?tx_outcome.envelope.to_canonical_bytes().len(), "envelope size");
            tracing::info!(target: "node::server", len=?tx_outcome.to_canonical_bytes().len(), "tx outcome size");
            let tx = TxAttestation::create(tx_outcome, sk);
            cmd_handle.publish_tx(tx.to_canonical_bytes())?;
        },
        SubmitTxResponse::Exec {receipt, ..} => {
            let tx_outcome = TxOutcome {
                envelope: tx_envelope,
                receipt_opt: Some(receipt),
            };
            tracing::info!(target: "node::server", len=?tx_outcome.envelope.to_canonical_bytes().len(), "envelope size");
            tracing::info!(target: "node::server", len=?tx_outcome.to_canonical_bytes().len(), "tx outcome size");
            let tx = TxAttestation::create(tx_outcome, sk);
            cmd_handle.publish_tx(tx.to_canonical_bytes())?;
        }
    };
    Ok(Json(intent))
}

pub fn handle_intent(db_handle: DBHandle, intent: &TxIntent) -> Result<SubmitTxResponse> {
    let payload = &intent.payload;
    match payload {
        TxPayload::Deploy{ image_id, elf, elf_hash } => {
            handle_deploy_tx(db_handle, intent, image_id, elf, elf_hash)
        },
        TxPayload::Exec { ctr_addr, input} => {
            handle_exec_tx(db_handle, intent, ctr_addr, input)
        }
    }
}

/// 验证交易签名
pub fn verify_tx_sig(envelope: &TxEnvelope) -> Result<()> {
    let vk = &envelope.intent.verifying_key;
    let intent_id_hash = envelope.intent.tx_id().0;
    vk.verify(&intent_id_hash, &envelope.signature)?;
    Ok(())
}

pub fn handle_deploy_tx(db_handle: DBHandle, intent: &TxIntent, image_id: &Digest, elf: &Vec<u8>, elf_hash: &Hash32) -> Result<SubmitTxResponse> {
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
    let ctr = Contract::create(intent.chain_id, elf_hash, image_id,
                               &intent.verifying_key, intent.nonce);
    // 获取合约Bech32m编码
    let ctr_bech32m =  ctr.addr.to_bech32m()?;
    tracing::info!("{}", ctr_bech32m);
    let ctr_addr_bytes = ctr.addr.to_bytes();
    
    Ok(SubmitTxResponse::Deploy {
        ctr_addr_bytes,
        image_id: computed_image_id,
        elf_hash: computed_elf_hash
    })
}

pub fn handle_exec_tx(db_handle: DBHandle, intent: &TxIntent, ctr_addr_bytes: &ChainAddrBytes, input: &Vec<u8>) -> Result<SubmitTxResponse> {
    tracing::info!(target: "node::handle", tx_id=?intent.tx_id());
    // 根据合约地址查找合约 BCS 编码向量
    let ctr = match db_handle.load_contract(ctr_addr_bytes)? {
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

    // 搭建虚拟机环境传入 input
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build().map_err(ServerError::ExecutorEnvBuild)?;

    // 根据虚拟机环境和 ELF 文件生成证明
    let prover = default_prover();
    let proof = prover.prove(env, &elf).map_err(ServerError::ProofGenerate)?;
    tracing::info!(target: "node::server", ?proof);
    let receipt = proof.receipt;
    let output: Vec<u8> = receipt.journal.decode().unwrap();
    tracing::info!(target: "node::server", ?output);
    Ok(SubmitTxResponse::Exec {
        ctr_addr_bytes: ctr_addr_bytes.clone(),
        image_id,
        elf_hash,
        input: input.clone(),
        receipt
    })
}
