use serde::{Deserialize, Serialize};
use primitives::hash::Hash32;
use tx::tx_envelope::TxEnvelope;
use crate::error::Result;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub parent_hash: Hash32,
    pub height: u128,
    pub tx_root: Hash32,
    pub timestamp: u128,
}


pub struct Block {
    header: BlockHeader,
    txs: Vec<TxEnvelope>
}

impl Block {
    pub fn new(header: BlockHeader, txs: Vec<TxEnvelope>) -> Self {
        Self {
            header,
            txs
        }
    }

    pub fn push_tx(&mut self, tx_envelope: TxEnvelope) -> Result<()> {
        self.txs.push(tx_envelope);
        Ok(())
    }

    pub fn block_id() {

    }
}