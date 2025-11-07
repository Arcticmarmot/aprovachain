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
use contract::contract::Contract;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SubmitTxResponse {
    Deploy {
        image_id: Digest,
        elf_hash: Hash32
    },
    Exec {
        image_id: Digest,
    }
}

/// 交易提交处理函数
pub async fn submit_tx(tx_bytes: Bytes) -> ApiResult<SubmitTxResponse> {
    // 从字节数组构造 TxEnvelope
    let tx_envelope_wire: TxEnvelopeWire = TxEnvelopeWire::try_from_bcs_bytes(tx_bytes.as_ref())?;
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
            let computed_image_id = risc0_zkvm::compute_image_id(elf).map_err(NodeError::ImageIdCompute)?;
            if &computed_image_id != image_id {
                return Err(NodeError::ImageIdMismatch)
            }
            let computed_elf_hash = sha256(elf);
            if &computed_elf_hash != elf_hash {
                return Err(NodeError::ElfHashMismatch)
            }
            tracing::info!("Deploy ImageId: {:?}", image_id.as_words());
            tracing::info!("Deploy ElfHash: {:?}", elf_hash);
            let ctr = Contract::create(intent.chain_id, computed_elf_hash, computed_image_id,
                                       &intent.verifying_key, intent.nonce);
            tracing::info!("{:?}", ctr);
            tracing::info!("{}", ctr.addr.to_bech32m()?);
            let ctr_addr_str =  ctr.addr.to_bech32m()?;
            let _ = kv_put(ctr_addr_str.as_bytes(), &ctr.to_canonical_bytes());
            let _ = kv_put(&ctr.elf_hash, elf);
            Ok(SubmitTxResponse::Deploy {
                image_id: computed_image_id,
                elf_hash: computed_elf_hash
            })
        },
        TxPayload::Exec { image_id, input} => {
            let env = ExecutorEnv::builder()
                .write(&input)
                .unwrap()
                .build().map_err(NodeError::ExecutorEnvBuild)?;

            let prover = default_prover();
            let elf = match kv_get(image_id.as_ref())? {
                Some(elf) => elf,
                None => return Err(NodeError::ElfFileNotFound)
            };
            tracing::info!("Elf file len: {}", elf.len());
            let proof = prover.prove(env, &elf);
            tracing::info!("PROOF: {:?}", proof);
            Ok(SubmitTxResponse::Exec {
                image_id: image_id.clone(),
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
