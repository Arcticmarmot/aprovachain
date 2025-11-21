use serde::{Deserialize, Serialize};
use account::keypair::{AccountSignature, AccountSignatureBytes, AccountSigningKey};
use primitives::hash::{sha256};
use crate::error::TxError;
use serde_with::{serde_as, Bytes};
use crate::tx_intent::{TxIntent, TxIntentWire};
use crate::error::Result;
use crate::tx_id::TxEnvelopeId;

#[derive(Debug, Clone)]
pub struct TxEnvelope {
    pub intent: TxIntent,
    pub signature: AccountSignature,
}

impl TxEnvelope {
    pub fn create(intent: TxIntent, sk: AccountSigningKey) -> Self {
        // 使用私钥对 TxIntent 计算出的 tx_intent_id 进行签名
        let tx_intent_id = intent.tx_id();
        let signature: AccountSignature = sk.sign(&tx_intent_id.0);
        Self {
            intent,
            signature
        }
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        TxEnvelopeWire::from(self).encode_bcs()
    }

    pub fn tx_id(&self) -> TxEnvelopeId {
        TxEnvelopeId::new(sha256(self.to_canonical_bytes()))
    }
}

impl TryFrom<TxEnvelopeWire> for TxEnvelope {
    type Error = TxError;

    fn try_from(wire: TxEnvelopeWire) -> Result<Self> {
        let intent = TxIntent::try_from(wire.intent)?;
        let signature = AccountSignature::from_bytes(&wire.signature);
        Ok(Self {
            intent,
            signature
        })
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxEnvelopeWire {
    pub intent: TxIntentWire,
    #[serde_as(as = "Bytes")]
    pub signature: AccountSignatureBytes
}

impl From<&TxEnvelope> for TxEnvelopeWire {
    fn from(envelope: &TxEnvelope) -> Self {
        let intent = &envelope.intent;
        Self {
            intent: intent.into(),
            signature: envelope.signature.to_bytes()
        }
    }
}


impl TxEnvelopeWire {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}

