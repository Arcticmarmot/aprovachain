use primitives::hash::Hash32;
use serde::{Deserialize, Serialize};
use crate::error::Result;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Agreement {
    AppendAck {
        height: u128,
        block_hash: Hash32,
        ack: bool,
    },
    ProposeBlock {
        height: u128,
        block_hash: Hash32,
        block_bytes: Vec<u8>
    },
    CommitBlock {
        height: u128,
        block_hash: Hash32,
    }
}

impl Agreement {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(bytes: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(bytes)?)
    }
}