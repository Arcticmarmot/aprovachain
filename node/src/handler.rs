use tx::tx_envelope::{TxEnvelopeWire, TxEnvelope};
use axum::body::Bytes;
use db::controller::{kv_get, kv_put};
use tx::tx_intent::{TxIntent, TxPayload};
use crate::error::{Result};
use primitives::hash::sha256;

/// 交易提交处理函数
pub async fn submit_tx(tx_bytes: Bytes) -> Result<String> {
    let tx_envelope_wire: TxEnvelopeWire = TxEnvelopeWire::try_from_bcs_bytes(tx_bytes.as_ref())?;
    let tx_envelope = TxEnvelope::try_from(tx_envelope_wire)?;
    println!("{:?}", tx_envelope);
    tx_handler(&tx_envelope.intent);
    let verify_result = verify_tx_sig(&tx_envelope);
    match verify_result {
        Ok(()) => Ok("submitted and verified success".to_string()),
        Err(e) => Ok("invalid transaction".to_string())
    }
}

/// 验证交易签名
pub fn verify_tx_sig(envelope: &TxEnvelope) -> Result<()> {
    let vk = &envelope.intent.verifying_key;
    let intent_id_hash = envelope.intent.tx_intent_id().0;
    vk.verify(&intent_id_hash, &envelope.signature)?;
    Ok(())
}

pub fn tx_handler(intent: &TxIntent) {
    let payload = &intent.payload;
    match payload {
        TxPayload::Deploy{ source } => {
            let image_id = sha256(source);
            let result = kv_put(&image_id, source);
            let result = kv_get(&image_id);
        },
        TxPayload::Exec { image_id, input} => {

        }
    }
}