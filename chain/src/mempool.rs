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

    pub fn push_tx(&mut self, tx_exec_seal: TxExecSeal) {
        self.txs.insert(tx_exec_seal);
    }

    pub fn count(&self) -> usize {
        self.txs.len()
    }
}

pub fn pack_block(mempool: &Mempool, parent: &BlockHeader, count: usize) -> Result<Block> {
    // 选出前 10 个交易打包进区块
    let mut txs: Vec<TxExecSeal> = mempool.txs.iter().cloned().collect();
    txs.sort_by_key(|tx| tx.tx_id.clone());
    let candidate_txs: Vec<&TxExecSeal> = txs.iter()
        .take(count)
        .collect();
    let timestamp = unix_time_millis()?;
    let candidate_txs_wire: Vec<TxExecSealWire> = candidate_txs.iter()
        .map(|&tx| TxExecSealWire::from(tx))
        .collect();

    let header = BlockHeader {
        parent_hash: parent.tx_root,
        height: parent.height + 1,
        tx_root: merkel_root(&candidate_txs),
        timestamp
    };
    Ok(Block::new(header, candidate_txs_wire))
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






