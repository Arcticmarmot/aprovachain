use risc0_zkvm::Receipt;
use serde::{Deserialize, Serialize};
use primitives::hash::{sha256};
use crate::envelope::{TxEnvelope, TxEnvelopeWire};
use crate::error::{Result, TxError};
use crate::id::TxOutcomeId;


#[derive(Debug, Clone)]
pub struct TxOutcome {
    pub envelope: TxEnvelope,
    pub receipt_opt: Option<Receipt>,
}

impl TryFrom<TxOutcomeWire> for TxOutcome {
    type Error = TxError;
    fn try_from(wire: TxOutcomeWire) -> Result<Self> {
        let envelope = TxEnvelope::try_from(wire.envelope)?;
        Ok(Self {
            envelope,
            receipt_opt: wire.receipt_opt
        })
    }
}

impl TxOutcome {
    pub fn create(envelope: TxEnvelope, receipt_opt: Option<Receipt>) -> Self { 
        Self {
            envelope,
            receipt_opt,
        }
    }
    
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        TxOutcomeWire::from(self).encode_bcs()
    }

    pub fn tx_id(&self) -> TxOutcomeId {
        TxOutcomeId::new(sha256(self.to_canonical_bytes()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutcomeWire {
    pub envelope: TxEnvelopeWire,
    pub receipt_opt: Option<Receipt>,
}

impl From<&TxOutcome> for TxOutcomeWire {
    fn from(outcome: &TxOutcome) -> Self {
        Self {
            envelope: TxEnvelopeWire::from(&outcome.envelope),
            receipt_opt: outcome.receipt_opt.clone(),
        }
    }
}

impl TxOutcomeWire {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(&self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(bytes: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(bytes)?)
    }
}
