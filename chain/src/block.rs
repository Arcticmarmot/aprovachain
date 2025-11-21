use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use primitives::clock::unix_time_millis;
use primitives::hash::Hash32;
use tx::tx_exec_seal::{TxExecSeal, TxExecSealWire};
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
}


#[derive(Debug)]
pub struct Block {
    pub header: BlockHeader,
    pub txs: HashSet<TxExecSeal>
}



impl Block {
    pub fn new(header: BlockHeader, txs: HashSet<TxExecSeal>) -> Self {
        Self {
            header,
            txs
        }
    }

    pub fn genesis() -> Result<Self> {
        let header = BlockHeader::genesis()?;
        let txs = HashSet::new();
        Ok(Self {
            header,
            txs
        })
    }

    pub fn push_tx(&mut self, tx_seal: TxExecSeal) -> Result<()> {
        self.txs.insert(tx_seal);
        Ok(())
    }

    pub fn wrap_block(&mut self) -> Result<Self> {
        Ok(Self {
            header: self.header,
            txs: self.txs.iter().cloned().collect::<HashSet<TxExecSeal>>()
        })
    }

    pub fn count(&self) -> usize {
        self.txs.len()
    }

    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        BlockWire::from(self).encode_bcs()
    }

}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockWire {
    pub header: BlockHeader,
    pub txs: Vec<TxExecSealWire>
}

impl From<&Block> for BlockWire {
    fn from(block: &Block) -> Self {
        let txs = block.txs.iter().map(|tx| {
            TxExecSealWire::try_from(tx).expect("tx convert wire")
        }).collect();
        Self {
            header: block.header,
            txs
        }
    }
}

impl BlockWire {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(bytes: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(bytes)?)
    }
}