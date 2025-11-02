use tx::tx_envelope::{TxEnvelopeWire, TxEnvelope};
use axum::{
    body::Bytes,
};
use crate::error::{NodeError, Result};

pub async fn submit_tx(tx_bytes: Bytes) -> Result<String> {
    let tx_envelope_wire: TxEnvelopeWire = TxEnvelopeWire::try_from_bcs_bytes(tx_bytes.as_ref())?;
    let tx_envelope = TxEnvelope::try_from(tx_envelope_wire)?;
    println!("{:?}", tx_envelope);
    let verify_result = verify_tx_sig(&tx_envelope);
    println!("{:?}", verify_result);
    Ok("submit success".to_string())
}

pub fn verify_tx_sig(envelope: &TxEnvelope) -> Result<()> {
    let vk = &envelope.intent.verifying_key;
    let intent_id_hash = envelope.intent.tx_intent_id().0;
    vk.verify(&intent_id_hash, &envelope.signature)?;
    Ok(())
}