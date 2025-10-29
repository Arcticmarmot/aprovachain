use rand_core::{TryRngCore};
use serde::{Deserialize, Serialize};
use account::keypair::{AccountSignature, AccountSignatureBytes, AccountSigningKey};
use primitives::hash::{sha256, Hash32};
use crate::error::TxError;
use serde_with::{serde_as, Bytes, hex::Hex};
use crate::tx_intent::{TxIntent, TxIntentWire};

pub type Result<T> = std::result::Result<T, TxError>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TxId(pub Hash32);


#[derive(Debug)]
pub struct TxEnvelope {
    pub intent: TxIntent,
    pub signature: AccountSignature,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct TxEnvelopWire {
    pub intent: TxIntentWire,
    #[serde_as(as = "Bytes")]
    pub signature: AccountSignatureBytes
}

impl From<&TxEnvelope> for TxEnvelopWire {
    fn from(envelope: &TxEnvelope) -> Self {
        let intent = &envelope.intent;
        Self {
            intent: intent.into(),
            signature: envelope.signature.to_bytes()
        }
    }
}

impl TxEnvelope {
    pub fn create(intent: TxIntent, sk: AccountSigningKey) -> Self {
        /// 使用私钥对 TxIntent 计算出的 tx_intent_id 进行签名
        let tx_intent_id = intent.tx_intent_id();
        let signature: AccountSignature = sk.sign(&tx_intent_id.0);
        Self {
            intent,
            signature
        }
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let envelop_wire = TxEnvelopWire::from(self);
        bcs::to_bytes(&envelop_wire).expect("BCS should be infallible by design")
    }

    pub fn tx_id(&self) -> TxId {
        TxId(sha256(self.to_canonical_bytes()))
    }
}
