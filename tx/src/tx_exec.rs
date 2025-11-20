use risc0_zkvm::Receipt;
use serde::{Deserialize, Serialize};
use account::keypair::{AccountVerifyingKey};
use primitives::hash::{sha256};
use crate::tx_envelope::{TxEnvelope, TxEnvelopeWire};
use crate::error::{Result, TxError};
use crate::tx_id::TxExecId;

pub struct TxExec {
    pub envelope: TxEnvelope,
    pub receipt: Receipt,
}

impl TryFrom<TxExecWire> for TxExec {
    type Error = TxError;
    fn try_from(wire: TxExecWire) -> Result<Self> {
        let envelope = TxEnvelope::try_from(wire.envelope)?;
        Ok(Self {
            envelope,
            receipt: wire.receipt
        })
    }
}

impl TxExec {
    pub fn create(envelope: TxEnvelope, receipt: Receipt, vk: AccountVerifyingKey) -> Result<Self> {
        Ok(Self {
            envelope,
            receipt,
        })
    }
    
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        TxExecWire::from(self).encode_bcs()
    }

    pub fn tx_id(&self) -> TxExecId {
        TxExecId::new(sha256(self.to_canonical_bytes()))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TxExecWire {
    envelope: TxEnvelopeWire,
    receipt: Receipt,
}

impl From<&TxExec> for TxExecWire {
    fn from(exec: &TxExec) -> Self {
        Self {
            envelope: TxEnvelopeWire::from(&exec.envelope),
            receipt: exec.receipt.clone(),
        }
    }
}

impl TxExecWire {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(&self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(bytes: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(bytes)?)
    }
}
