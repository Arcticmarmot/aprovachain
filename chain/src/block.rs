use serde::{Deserialize, Serialize};
use primitives::clock::unix_time_millis;
use primitives::hash::Hash32;
use tx::tx_exec_seal::TxExecSeal;
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
    pub txs: Vec<TxExecSeal>
}

impl Block {
    pub fn new(header: BlockHeader, txs: Vec<TxExecSeal>) -> Self {
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

    pub fn push_tx(&mut self, tx_seal: TxExecSeal) -> Result<()> {
        self.txs.push(tx_seal);
        Ok(())
    }

}