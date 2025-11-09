use tx::tx_envelope::{TxEnvelopeWire, TxEnvelope};
use axum::body::{Body, Bytes};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, StatusCode};
use axum::Json;
use risc0_zkvm::{default_prover, Digest, ExecutorEnv, Prover};
use db::controller::{kv_get, kv_put};
use tx::tx_intent::{TxIntent, TxPayload};
use crate::error::{ApiResult, NodeError, Result};
use primitives::hash::{sha256, Hash32};
use serde::{Deserialize, Serialize};
use account::address::ChainAddrBytes;
use contract::contract::{Contract, ContractWire};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SubmitTxResponse {
    Deploy {
        image_id: Digest,
        elf_hash: Hash32
    },
    Exec {
        ctr_addr: ChainAddrBytes,
    }
}

/// 交易提交处理函数
pub async fn submit_tx(tx_bytes: Bytes) -> ApiResult<SubmitTxResponse> {
    // 从字节数组构造 TxEnvelope
    let tx_envelope_wire: TxEnvelopeWire = TxEnvelopeWire::try_decode_bcs(tx_bytes.as_ref())?;
    let tx_envelope = TxEnvelope::try_from(tx_envelope_wire)?;
    // tracing::debug!("{:?}", tx_envelope);
    // 验证交易签名是否有效
    let _ = verify_tx_sig(&tx_envelope)?;
    // 执行交易
    let intent = handle_intent(&tx_envelope.intent)?;
    Ok(Json(intent))
}

pub fn handle_intent(intent: &TxIntent) -> Result<SubmitTxResponse> {
    let payload = &intent.payload;
    match payload {
        TxPayload::Deploy{ image_id, elf, elf_hash } => {
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
            let ctr_addr =  ctr.addr.to_bech32m()?;
            tracing::info!("{}", ctr_addr);
            tracing::info!("{:?}", ctr.addr.to_bytes());

            // key: 合约的 addr 字节数组
            // value: 合约的BCS编码
            let _ = kv_put(&ctr.addr.to_bytes(), &ctr.to_canonical_bytes());
            // key: ELF文件哈希
            // value: ELF文件字节数组
            let _ = kv_put(&ctr.elf_hash, elf);

            Ok(SubmitTxResponse::Deploy {
                image_id: computed_image_id,
                elf_hash: computed_elf_hash
            })
        },
        TxPayload::Exec { ctr_addr, input} => {
            let env = ExecutorEnv::builder()
                .write(&input)
                .unwrap()
                .build().map_err(NodeError::ExecutorEnvBuild)?;

            let prover = default_prover();

            let elf_hash = match kv_get(ctr_addr)? {
                Some(ctr_bytes) => {
                    let wire = ContractWire::try_encode_bcs(&ctr_bytes)?;
                    let ctr = Contract::try_from(wire)?;
                    ctr.elf_hash
                },
                None => return Err(NodeError::ElfFileNotFound)
            };
            tracing::info!("elf_hash: {:?}", elf_hash);

            let elf = match kv_get(&elf_hash)? {
                Some(elf) => elf,
                None => return Err(NodeError::ElfFileNotFound)
            };
            tracing::info!("Elf file len: {}", elf.len());
            let proof = prover.prove(env, &elf);
            tracing::info!("PROOF: {:?}", proof);
            // TODO: 返回有效 Response
            Ok(SubmitTxResponse::Exec {
                ctr_addr: ctr_addr.clone()
            })
        }
    }
}

/// 验证交易签名
pub fn verify_tx_sig(envelope: &TxEnvelope) -> Result<()> {
    let vk = &envelope.intent.verifying_key;
    let intent_id_hash = envelope.intent.tx_intent_id().0;
    vk.verify(&intent_id_hash, &envelope.signature)?;
    Ok(())
}
