use ed25519_dalek::{Signature, VerifyingKey};
use rand_core::{TryRngCore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use account::address::{AccountAddress, UserAddress};
use account::keypair::{AccountSigningKey, AccountVerifyingKey};
use primitives::rand::random_u128;
use primitives::clock::unix_time_secs;
use crate::error::TxError;

pub type Result<T> = std::result::Result<T, TxError>;
pub type Hash32 = [u8; 32];
pub type Sig64 = [u8; 64];

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

#[derive(Debug)]
pub struct TxEnvelope {
    pub intent: TxIntent,
    pub signature: Signature,
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

    pub fn intent_id(&self) -> Hash32 {
        let intent_wire = TxIntentWire::from(self);
        let data = bcs::to_bytes(&intent_wire).unwrap();
        let h = Sha256::digest(&data);
        let mut out = [0u8; 32];
        out.copy_from_slice(&h);
        out
    }
}

impl TxIntentWire {
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

impl TxEnvelope {
    // pub fn new(intent: TxIntent, sk: AccountSigningKey) -> Self {
    //     Self {
    //
    //     }
    // }
}
