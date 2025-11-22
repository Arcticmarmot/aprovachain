use std::cmp::min;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use primitives::clock::unix_time_millis;
use primitives::hash::Hash32;
use tx::tx_exec_seal::{TxExecSeal, TxExecSealWire};
use tx::tx_id::TxExecSealId;
use crate::block::{Block, BlockHeader};
use crate::error::Result;
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct Mempool {
    pub txs: HashSet<TxExecSeal>
}

impl Debug for Mempool {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Mempool tx_ids:")?;
        for tx in &self.txs {
            writeln!(f, "{:?}", tx.tx_id)?;
        }
        Ok(())
    }
}

impl Mempool {
    pub fn new() -> Self {
        Self {
            txs: HashSet::new()
        }
    }

    pub fn push_tx(&mut self, tx_bytes: Vec<u8>) -> Result<()> {
        let wire = TxExecSealWire::try_decode_bcs(&tx_bytes)?;
        let tx = TxExecSeal::try_from(wire)?;
        self.txs.insert(tx);
        Ok(())
    }

    pub fn remove_tx(&mut self, tx: &TxExecSeal) -> bool {
        self.txs.remove(tx)
    }

    pub fn count(&self) -> usize {
        self.txs.len()
    }
}


pub struct MempoolHandle {
    mempool: Mempool,
    bufpool: Vec<TxExecSeal>
}

impl MempoolHandle {
    pub fn new() -> Self {
        Self {
            mempool: Mempool::new(),
            bufpool: Vec::new()
        }
    }

    pub fn received_tx(&mut self, tx_bytes: Vec<u8>) -> Result<()> {
        self.mempool.push_tx(tx_bytes)?;
        Ok(())
    }

    pub fn pack_block(&mut self, parent: &BlockHeader, count: usize) -> Result<Block> {
        if self.mempool.count() == 0 {
            let empty_block = Block::empty(parent)?;
            return Ok(empty_block)
        }
        let count = min(count, self.mempool.count());
        // 选出前 count 个交易打包进区块
        let mut txs: Vec<TxExecSeal> = self.mempool.txs.iter().cloned().collect();
        txs.sort_by_key(|tx| tx.tx_id.clone());

        let candidate_txs: Vec<&TxExecSeal> = txs.iter()
            .take(count)
            .collect();
        let candidate_txs_wire: Vec<TxExecSealWire> = candidate_txs.iter()
            .map(|&tx| TxExecSealWire::from(tx))
            .collect();
        for &tx in &candidate_txs {
            if self.mempool.remove_tx(tx) {
                self.bufpool.push(tx.clone())
            }
        }
        let timestamp = unix_time_millis()?;

        let header = BlockHeader {
            parent_hash: parent.hash(),
            height: parent.height + 1,
            tx_root: merkel_root(&candidate_txs),
            timestamp
        };
        Ok(Block::new(header, candidate_txs_wire))
    }
}

pub fn merkel_root(txs: &Vec<&TxExecSeal>) -> Hash32 {
    let mut queue: Vec<TxExecSealId> = txs.iter()
        .map(|tx| tx.tx_id.clone())
        .collect();
    while queue.len() > 1 {
        let mut next = Vec::with_capacity((queue.len() + 1) / 2);
        for pair in queue.chunks(2) {
            let node = if pair.len() == 2 {
                let mut hasher = Sha256::new();
                hasher.update(&pair[0].0);
                hasher.update(&pair[1].0);
                TxExecSealId::new(hasher.finalize().into())
            } else {
                let mut hasher = Sha256::new();
                hasher.update(&pair[0].0);
                TxExecSealId::new(hasher.finalize().into())
            };
            next.push(node);
        }
        queue = next;
    }
    queue[0].0
}