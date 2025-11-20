use tx::tx_envelope::{TxEnvelopeWire, TxEnvelope};
use axum::body::{Bytes};
use axum::extract::State;
use axum::Json;
use risc0_zkvm::{default_prover, Digest, ExecutorEnv, Prover, Receipt};
use db::controller::{kv_get, kv_put};
use tx::tx_intent::{TxIntent, TxPayload};
use crate::error::{ApiResult, NodeError, Result};
use primitives::hash::{sha256, Hash32};
use serde::{Deserialize, Serialize};
use account::address::ChainAddrBytes;
use contract::contract::{Contract, ContractWire};
use tx::tx_exec::{TxExec, TxExecWire};
use tx::tx_exec_seal::TxExecSeal;
use crate::context::{AppState, SubmitTxResponse};

/// 交易提交处理函数
pub async fn submit_tx(State(state) : State<AppState>, tx_bytes: Bytes) -> ApiResult<SubmitTxResponse> {
    // 从字节数组构造 TxEnvelope
    let tx_envelope_wire: TxEnvelopeWire = TxEnvelopeWire::try_decode_bcs(tx_bytes.as_ref())?;
    let tx_envelope = TxEnvelope::try_from(tx_envelope_wire)?;
    tracing::info!(target:"node::axum", tx_envelope_id=%tx_envelope.tx_id(), "tx envelope id");
    // 验证交易签名是否有效
    // TODO: 重放交易攻击，拒绝重复的 nonce
    verify_tx_sig(&tx_envelope)?;
    let p2p_handle = state.p2p_handle;
    let sk = state.sk;
    // 执行交易
    let intent = handle_intent(&tx_envelope.intent)?;
    match intent.clone() {
        SubmitTxResponse::Deploy{ ctr_addr, image_id, elf_hash } => {
        },
        SubmitTxResponse::Exec {ctr_addr, image_id, elf_hash, input, receipt} => {
            let tx_exec = TxExec {
                envelope: tx_envelope,
                receipt,
            };
            let wire = TxExecWire::from(&tx_exec);
            tracing::info!(target: "node::axum", len=?tx_exec.envelope.to_canonical_bytes().len(), "envelope size");
            tracing::info!(target: "node::axum", len=?tx_exec.to_canonical_bytes().len(), "tx exec size");
            let tx_seal = TxExecSeal::create(tx_exec, sk);
            p2p_handle.publish_tx(tx_seal.to_canonical_bytes())?;
        }
    };
    Ok(Json(intent))
}

pub fn handle_intent(intent: &TxIntent) -> Result<SubmitTxResponse> {
    let payload = &intent.payload;
    match payload {
        TxPayload::Deploy{ image_id, elf, elf_hash } => {
            handle_deploy_tx(intent, image_id, elf, elf_hash)
        },
        TxPayload::Exec { ctr_addr, input} => {
            handle_exec_tx(intent, ctr_addr, input)
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

pub fn handle_deploy_tx(intent: &TxIntent, image_id: &Digest, elf: &Vec<u8>, elf_hash: &Hash32) -> Result<SubmitTxResponse> {
    // 验证 ELF 文件哈希是否对应
    let computed_elf_hash = sha256(elf);
    if &computed_elf_hash != elf_hash {
        return Err(NodeError::ElfHashMismatch)
    }
    // 验证 image_id 是否对应
    let computed_image_id = risc0_zkvm::compute_image_id(elf).map_err(NodeError::ImageIdCompute)?;
    if &computed_image_id != image_id {
        return Err(NodeError::ImageIdMismatch)
    }
    tracing::info!("Deploy ImageId: {:?}", image_id.as_words());
    tracing::info!("Deploy ElfHash: {:?}", elf_hash);

    // 创建合约
    let ctr = Contract::create(intent.chain_id, computed_elf_hash, computed_image_id,
                               &intent.verifying_key, intent.nonce);
    // 获取合约Bech32m编码
    let ctr_bech32m =  ctr.addr.to_bech32m()?;
    tracing::info!("{}", ctr_bech32m);
    let ctr_addr = ctr.addr.to_bytes();
    tracing::info!("{:?}", ctr_addr);

    // key: 合约的 addr 字节数组
    // value: 合约的BCS编码
    let _ = kv_put(&ctr_addr, &ctr.to_canonical_bytes());
    // key: ELF文件哈希
    // value: ELF文件字节数组
    let _ = kv_put(&ctr.elf_hash, elf);

    Ok(SubmitTxResponse::Deploy {
        ctr_addr,
        image_id: computed_image_id,
        elf_hash: computed_elf_hash
    })
}

pub fn handle_exec_tx(intent: &TxIntent, ctr_addr: &ChainAddrBytes, input: &Vec<u8>) -> Result<SubmitTxResponse> {
    // 根据合约地址查找合约 BCS 编码向量
    let ctr = match kv_get(ctr_addr)? {
        Some(ctr_bytes) => {
            let wire = ContractWire::try_encode_bcs(&ctr_bytes)?;
            let ctr = Contract::try_from(wire)?;
            ctr
        },
        None => return Err(NodeError::ContractNotFound)
    };
    tracing::info!("Contract: {:?}", ctr);

    let image_id = ctr.image_id;
    // 从合约中获取 elf_hash
    let elf_hash = ctr.elf_hash;

    // 根据 ELF 文件哈希查找 ELF 文件
    let elf = match kv_get(&elf_hash)? {
        Some(elf) => elf,
        None => return Err(NodeError::ElfFileNotFound)
    };
    tracing::info!("Elf file len: {}", elf.len());

    // 搭建虚拟机环境传入 input
    let env = ExecutorEnv::builder()
        .write(&input)
        .unwrap()
        .build().map_err(NodeError::ExecutorEnvBuild)?;

    // 根据虚拟机环境和 ELF 文件生成证明
    let prover = default_prover();
    let proof = prover.prove(env, &elf).map_err(NodeError::ProofGenerate)?;
    tracing::info!("PROOF: {:?}", proof);
    let receipt = proof.receipt;
    let output: Vec<u8> = receipt.journal.decode().unwrap();
    tracing::info!("output: {:?}", output);
    Ok(SubmitTxResponse::Exec {
        ctr_addr: ctr_addr.clone(),
        image_id,
        elf_hash,
        input: input.clone(),
        receipt
    })
}
