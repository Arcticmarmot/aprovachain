use std::fmt::{Display, Formatter};
use std::str::FromStr;
use risc0_zkvm::Digest;
use serde::{Deserialize, Serialize};
use account::address::{AddressBytes, ChainAddress};
use account::keypair::{AccountVerifyingKey, AccountVerifyingKeyBytes};
use chain::spec::{ChainId};
use primitives::rand::random_u128;
use primitives::clock::unix_time_millis;
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
        let s = s.strip_prefix("0x").ok_or(TxError::TxIntentIdPrefix)?;
        let mut out = [0u8; 32];
        hex::decode_to_slice(s, &mut out)?;
        Ok(TxIntentId(out))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TxPayload {
    Exec {
        image_id: Digest,
        input: Vec<u8>
    },
    Deploy {
        image_id: Digest,
        elf: Vec<u8>,
        elf_hash: Hash32
    },
}

#[derive(Debug)]
pub struct TxIntent {
    pub chain_id: ChainId,
    pub nonce: u128,
    pub address: ChainAddress,
    pub verifying_key: AccountVerifyingKey,
    pub timestamp: u128,
    pub payload: TxPayload,
}

impl TryFrom<TxIntentWire> for TxIntent {
    type Error = TxError;

    fn try_from(wire: TxIntentWire) -> Result<Self> {
        let chain_id = ChainId(wire.chain_id);
        let address = ChainAddress::create_from_bytes(chain_id, wire.address);
        let verifying_key = AccountVerifyingKey::from_bytes(&wire.verifying_key)?;
        Ok(Self {
            chain_id,
            nonce: wire.nonce,
            address,
            verifying_key,
            timestamp: wire.timestamp,
            payload: wire.payload
        })
    }
}


impl TxIntent {
    pub fn create(chain_id: ChainId, addr: ChainAddress, vk: AccountVerifyingKey, payload: TxPayload) -> Result<Self> {
        let nonce = random_u128()?;
        let timestamp = unix_time_millis()?;
        Ok(Self {
            chain_id,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TxIntentWire {
    pub chain_id: u64,
    pub nonce: u128,
    pub address: AddressBytes,
    pub verifying_key: AccountVerifyingKeyBytes,
    pub timestamp: u128,
    pub payload: TxPayload
}


impl TxIntentWire {
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

impl From<&TxIntent> for TxIntentWire {
    fn from(intent: &TxIntent) -> Self {
        Self {
            chain_id: intent.chain_id.0,
            nonce: intent.nonce,
            address: intent.address.to_bytes(),
            verifying_key: intent.verifying_key.to_bytes(),
            timestamp: intent.timestamp,
            payload: intent.payload.clone()
        }
    }
}
