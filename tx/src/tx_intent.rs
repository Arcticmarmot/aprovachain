use std::fmt::{Display, Formatter};
use std::str::FromStr;
use rand_core::{TryRngCore};
use serde::{Deserialize, Serialize};
use account::address::{AccountAddress, UserAddress, AccountAddressBytes};
use account::keypair::{AccountVerifyingKey, AccountVerifyingKeyBytes};
use primitives::rand::random_u128;
use primitives::clock::unix_time_secs;
use primitives::hash::{sha256, Hash32};
use crate::error::TxError;

pub type Result<T> = std::result::Result<T, TxError>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TxIntentId(pub Hash32);

impl Display for TxIntentId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "0x{}", hex::encode(&self.0))
    }
}

impl FromStr for TxIntentId {
    type Err = TxError;

    fn from_str(s: &str) -> Result<Self> {
        let s = s.strip_prefix("0x").unwrap_or(s);
        let mut out = [0u8; 32];
        hex::decode_to_slice(s, &mut out)?;
        Ok(TxIntentId(out))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TxPayload {
    Exec { image_id: String, input:Vec<u8> },
    Deploy { source: Vec<u8> },
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
pub struct TxIntentWire {
    pub nonce: u128,
    pub address: AccountAddressBytes,
    pub verifying_key: AccountVerifyingKeyBytes,
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
            address: intent.address.to_bytes(),
            verifying_key: intent.verifying_key.to_bytes(),
            timestamp: intent.timestamp,
            payload: intent.payload.clone()
        }
    }
}
