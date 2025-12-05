use std::fmt::{Debug, Formatter};
use risc0_zkvm::{Digest};
use serde::{Deserialize, Serialize};
use spec::chain::{ChainId};
use platform::rand::random_u128;
use platform::clock::unix_time_millis;
use primitives::hash::{sha256, Hash32};
use apps::ctr_io::AccessSet;
use crate::error::TxError;
use crate::error::Result;
use crate::id::{TxIntentId};

#[derive(Clone, Serialize, Deserialize)]
pub enum TxPayload {
    Exec {
        ctr_addr_str: String,
        input: Vec<u8>,
        access_set: AccessSet
    },
    Deploy {
        image_id: Digest,
        elf: Vec<u8>,
        elf_hash: Hash32
    },
    Update {
        ctr_addr_str: String,
        image_id: Digest,
        elf: Vec<u8>,
        elf_hash: Hash32
    }
}

impl Debug for TxPayload {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TxPayload {{ ")?;
        match self {
            TxPayload::Exec { ctr_addr_str, input, ..  } => {
                write!(f, "type: exec, ")?;
                write!(f, "ctr_addr: {}, ", ctr_addr_str)?;
                write!(f, "input: {:?}", input)?;
            },
            TxPayload::Deploy { image_id, elf_hash, .. } => {
                write!(f, "type: deploy, ")?;
                write!(f, "image_id: {}, ", image_id.to_string())?;
                write!(f, "elf_hash: {:?}, ", elf_hash)?;
            },
            TxPayload::Update { ctr_addr_str, image_id, elf_hash, ..  } => {
                write!(f, "type: update, ")?;
                write!(f, "ctr_addr: {}", ctr_addr_str)?;
                write!(f, "image_id: {}, ", image_id.to_string())?;
                write!(f, "elf_hash: {:?}, ", elf_hash)?;
            }
        }
        write!(f, " }}")
    }
}

#[derive(Debug, Clone)]
pub struct TxIntent {
    pub chain_id: ChainId,
    pub nonce: u128,
    pub timestamp: u128,
    pub payload: TxPayload,
}

impl TryFrom<TxIntentWire> for TxIntent {
    type Error = TxError;

    fn try_from(wire: TxIntentWire) -> Result<Self> {
        let chain_id = ChainId(wire.chain_id);
        Ok(Self {
            chain_id,
            nonce: wire.nonce,
            timestamp: wire.timestamp,
            payload: wire.payload
        })
    }
}


impl TxIntent {
    pub fn create(chain_id: ChainId, payload: TxPayload) -> Result<Self> {
        let nonce = random_u128()?;
        let timestamp = unix_time_millis()?;
        Ok(Self {
            chain_id,
            nonce,
            timestamp,
            payload,
        })
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        TxIntentWire::from(self).encode_bcs()
    }

    pub fn tx_id(&self) -> TxIntentId {
        TxIntentId::new(sha256(self.to_canonical_bytes()))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxIntentWire {
    pub chain_id: u64,
    pub nonce: u128,
    pub timestamp: u128,
    pub payload: TxPayload
}


impl TxIntentWire {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    /// NOTE: 输入为不可信的 BCS 编码字节切片；仅尝试解码
    pub fn try_decode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}

impl From<&TxIntent> for TxIntentWire {
    fn from(intent: &TxIntent) -> Self {
        Self {
            chain_id: intent.chain_id.0,
            nonce: intent.nonce,
            timestamp: intent.timestamp,
            payload: intent.payload.clone()
        }
    }
}
