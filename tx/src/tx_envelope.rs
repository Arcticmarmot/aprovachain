use rand_core::{TryRngCore};
use serde::{Deserialize, Serialize};
use account::address::{AccountAddress, UserAddress};
use account::keypair::{AccountSignature, AccountSignatureBytes, AccountSigningKey, AccountVerifyingKey};
use primitives::rand::random_u128;
use primitives::clock::unix_time_secs;
use primitives::hash::{sha256, Hash32};
use crate::error::TxError;
use serde_with::{serde_as, Bytes};

pub type Result<T> = std::result::Result<T, TxError>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TxIntentId(Hash32);
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TxId(Hash32);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TxPayload {
    Exec { image_id: String, input:Vec<u8> }
}

#[derive(Debug)]
pub struct TxIntent {
    pub nonce: u128,
    pub address: AccountAddress<UserAddress>,
    pub verifying_key: AccountVerifyingKey,
    pub timestamp: u64,
    pub payload: TxPayload,
}

#[derive(Debug, Serialize, Deserialize)]
struct TxIntentWire {
    pub nonce: u128,
    pub address: String,
    pub verifying_key: [u8; 32],
    pub timestamp: u64,
    pub payload: TxPayload
}

impl TxIntent {
    pub fn create(addr: AccountAddress<UserAddress>, vk: AccountVerifyingKey, payload: TxPayload) -> Result<Self> {
        let nonce = random_u128()?;
        let timestamp = unix_time_secs()?;
        Ok(Self {
            nonce,
            timestamp,
            address: addr,
            verifying_key: vk,
            payload,
        })
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let intent_wire = TxIntentWire::from(self);
        bcs::to_bytes(&intent_wire).expect("BCS should be infallible by design")
    }

    pub fn tx_intent_id(&self) -> TxIntentId {
        TxIntentId(sha256(self.to_canonical_bytes()))
    }
}

impl From<&TxIntent> for TxIntentWire {
    fn from(intent: &TxIntent) -> Self {
        Self {
            nonce: intent.nonce,
            address: intent.address.to_string(),
            verifying_key: intent.verifying_key.to_bytes(),
            timestamp: intent.timestamp,
            payload: intent.payload.clone()
        }
    }
}

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
        let signature = sk.sign(&tx_intent_id.0);
        println!("signature signed: {:?}", signature.to_bytes());
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
        println!("canonical: {:?}", self.to_canonical_bytes());
        TxId(sha256(self.to_canonical_bytes()))
    }
}
