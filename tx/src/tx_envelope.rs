use std::fmt::{Display, Formatter};
use std::str::FromStr;
use rand_core::{TryRngCore};
use serde::{Deserialize, Serialize};
use serde::__private226::de::IdentifierDeserializer;
use account::keypair::{AccountSignature, AccountSignatureBytes, AccountSigningKey};
use primitives::hash::{sha256, Hash32};
use crate::error::TxError;
use serde_with::{serde_as, Bytes};
use crate::tx_intent::{TxIntent, TxIntentWire};

pub type Result<T> = std::result::Result<T, TxError>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TxId(pub Hash32);

impl Display for TxId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "0x{}", hex::encode(&self.0))
    }
}

impl FromStr for TxId {
    type Err = TxError;

    fn from_str(s: &str) -> crate::tx_intent::Result<Self> {
        let s = s.strip_prefix("0x").ok_or(TxError::TxIdPrefix)?;
        let mut out = [0u8; 32];
        hex::decode_to_slice(s, &mut out)?;
        Ok(TxId(out))
    }
}

#[derive(Debug)]
pub struct TxEnvelope {
    pub intent: TxIntent,
    pub signature: AccountSignature,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct TxEnvelopeWire {
    pub intent: TxIntentWire,
    #[serde_as(as = "Bytes")]
    pub signature: AccountSignatureBytes
}

impl TxEnvelopeWire {
    pub fn to_bcs_bytes(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn from_bcs_bytes(b: &[u8]) -> Self {
        bcs::from_bytes(b).expect("BCS should be infallible by design")
    }

    pub fn try_from_bcs_bytes(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
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
        let envelop_wire = TxEnvelopeWire::from(self);
        bcs::to_bytes(&envelop_wire).expect("BCS should be infallible by design")
    }

    pub fn tx_id(&self) -> TxId {
        TxId(sha256(self.to_canonical_bytes()))
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
