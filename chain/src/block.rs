use serde::{Deserialize, Serialize};
use primitives::clock::unix_time_millis;
use primitives::hash::{sha256, Hash32, HASH32_ZERO};
use tx::tx_exec_seal::{TxExecSealWire};
use crate::error::Result;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub parent_hash: Hash32,
    pub height: u128,
    pub tx_root: Hash32,
    pub timestamp: u128,
}

impl BlockHeader {
    pub fn genesis() -> Result<Self> {
        let now = unix_time_millis()?;
        Ok(Self {
            parent_hash: [0u8; 32],
            height: 0,
            tx_root: [0u8; 32],
            timestamp: now
        })
    }

    pub fn try_decode_bcs(bytes: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(bytes)?)
    }

    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn hash(&self) -> Hash32 {
        sha256(self.encode_bcs())
    }

    pub fn child_of(parent: &BlockHeader, tx_root: Hash32) -> Result<Self> {
        let now = unix_time_millis()?;
        Ok(Self{
            parent_hash: parent.hash(),
            height: parent.height + 1,
            tx_root,
            timestamp: now
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<TxExecSealWire>
}

impl Block {
    pub fn new(header: BlockHeader, txs: Vec<TxExecSealWire>) -> Self {
        Self {
            header,
            txs
        }
    }

    pub fn genesis() -> Result<Self> {
        let header = BlockHeader::genesis()?;
        let txs = Vec::new();
        Ok(Self {
            header,
            txs
        })
    }

    pub fn empty(parent: &BlockHeader) -> Result<Self> {
        let header = BlockHeader::child_of(&parent, HASH32_ZERO)?;
        let txs = Vec::new();
        Ok(Self {
            header,
            txs
        })
    }

    pub fn try_decode_bcs(bytes: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(bytes)?)
    }
    
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }
}