use tx::tx_envelope::{TxEnvelopeWire, TxEnvelope};
use axum::body::Bytes;
use risc0_zkvm::{default_prover, ExecutorEnv, Prover};
use db::controller::{kv_get, kv_put};
use tx::tx_intent::{TxIntent, TxPayload};
use crate::error::{Result};
use primitives::hash::sha256;

/// 交易提交处理函数
pub async fn submit_tx(tx_bytes: Bytes) -> Result<String> {
    let tx_envelope_wire: TxEnvelopeWire = TxEnvelopeWire::try_from_bcs_bytes(tx_bytes.as_ref())?;
    let tx_envelope = TxEnvelope::try_from(tx_envelope_wire)?;
    // println!("{:?}", tx_envelope);
    let _ = tx_handler(&tx_envelope.intent);
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

pub fn tx_handler(intent: &TxIntent) -> Result<()> {
    let payload = &intent.payload;
    match payload {
        TxPayload::Deploy{ source } => {
            let image_id = sha256(source);
            tracing::info!("Deploy ImageId: {:?}", image_id);
            kv_put(&image_id, source)?;
        },
        TxPayload::Exec { image_id, input} => {
            let env = ExecutorEnv::builder()
                .write(&input)
                .unwrap()
                .build()
                .unwrap();

            let prover = default_prover();
            let elf_file = match kv_get(image_id)?{
                Some(elf) => elf,
                None => return Ok(())
            };

            tracing::info!("LEN: {}", elf_file.len());
            let proof = prover.prove(env, &elf_file);
            tracing::info!("PROOF: {:?}", proof);
        }
    }
    Ok(())
}