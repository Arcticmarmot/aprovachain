use tx::tx_envelope::{TxEnvelopeWire, TxEnvelope};
use axum::{
    body::Bytes,
};
use crate::error::NodeError;

type Result<T> = std::result::Result<T, NodeError>;
pub async fn submit_tx(tx_bytes: Bytes) -> Result<String> {
    let tx_envelope_wire: TxEnvelopeWire = TxEnvelopeWire::try_from_bcs_bytes(tx_bytes.as_ref())?;
    let tx_envelope = TxEnvelope::try_from(tx_envelope_wire)?;
    println!("{:?}", tx_envelope);
    Ok("submit success".to_string())
}