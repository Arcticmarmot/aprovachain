use std::cmp::min;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use platform::clock::unix_time_millis;
use primitives::hash::Hash32;
use tx::attestation::{TxAttestation, TxAttestationWire};
use tx::id::TxAttestationId;
use crate::block::{OrderedBlock, BlockHeader};
use crate::error::Result;
use sha2::{Digest, Sha256};
use tx::intent::TxPayload;

#[derive(Clone)]
pub struct Mempool {
    pub txs: HashSet<TxAttestation>
}

impl Debug for Mempool {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mempool tx_ids:")?;
        for tx in &self.txs {
            write!(f, "{:?}", tx.tx_id)?;
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
        let wire = TxAttestationWire::try_decode_bcs(&tx_bytes)?;
        let tx = TxAttestation::try_from(wire)?;
        self.txs.insert(tx);
        Ok(())
    }

    pub fn remove_tx(&mut self, tx: &TxAttestation) -> bool {
        self.txs.remove(tx)
    }

    pub fn count(&self) -> usize {
        self.txs.len()
    }
}

#[derive(Debug)]
pub struct MempoolHandle {
    mempool: Mempool,
    pending_txs: Vec<TxAttestation>
}

impl MempoolHandle {
    pub fn new() -> Self {
        Self {
            mempool: Mempool::new(),
            pending_txs: Vec::new()
        }
    }

    pub fn received_tx(&mut self, tx_bytes: Vec<u8>) -> Result<()> {
        self.mempool.push_tx(tx_bytes)?;
        Ok(())
    }

    pub fn pack_block(&mut self, parent: &BlockHeader, count: usize) -> Result<OrderedBlock> {
        // 1. mempool为空，出空块
        if self.mempool.count() == 0 {
            let empty_block = OrderedBlock::empty(parent)?;
            return Ok(empty_block)
        }
        // 2. 复制 + 排序 交易
        let mut txs: Vec<TxAttestation> = self.mempool.txs.iter().cloned().collect();
        txs.sort();

        let count = min(count, self.mempool.count());
        let mut candidate_ids: Vec<TxAttestationId> = Vec::with_capacity(count);
        let mut candidate_wires: Vec<TxAttestationWire> = Vec::with_capacity(count);
        for tx in txs.into_iter().take(count) {
            candidate_ids.push(tx.tx_id);
            candidate_wires.push(TxAttestationWire::from(&tx));
            if self.mempool.remove_tx(&tx) {
                self.pending_txs.push(tx);
            }
        }

        let tx_root = merkel_root(&candidate_ids);
        let timestamp = unix_time_millis()?;

        let header = BlockHeader {
            parent_hash: parent.hash(),
            height: parent.height + 1,
            tx_root,
            timestamp
        };
        Ok(OrderedBlock::new(header, candidate_wires))
    }

    pub fn simulate_pack_block(&mut self, parent: &BlockHeader, count: usize, simulate_size: usize) -> Result<OrderedBlock> {
        // 1. mempool为空，出空块
        if self.mempool.count() == 0 {
            let empty_block = OrderedBlock::empty(parent)?;
            return Ok(empty_block)
        }
        // 2. 复制 + 排序 交易
        let mut txs: Vec<TxAttestation> = self.mempool.txs.iter().cloned().collect();
        
        txs.sort();
        let tx = txs[0].clone();
        let (candidate_ids, candidate_wires) = match &tx.outcome.envelope.intent.payload {
            TxPayload::Exec { .. } => {
                let mut candidate_ids: Vec<TxAttestationId> = Vec::with_capacity(simulate_size);
                let mut candidate_wires: Vec<TxAttestationWire> = Vec::with_capacity(simulate_size);
                for _ in 0..simulate_size {
                    let tx = tx.clone();
                    candidate_ids.push(tx.tx_id);
                    candidate_wires.push(TxAttestationWire::from(&tx));
                    if self.mempool.remove_tx(&tx) {
                        self.pending_txs.push(tx);
                    }
                }
                (candidate_ids, candidate_wires)
            },
            _ => {
                let count = min(count, self.mempool.count());
                let mut candidate_ids: Vec<TxAttestationId> = Vec::with_capacity(count);
                let mut candidate_wires: Vec<TxAttestationWire> = Vec::with_capacity(count);
                for tx in txs.into_iter().take(count) {
                    candidate_ids.push(tx.tx_id);
                    candidate_wires.push(TxAttestationWire::from(&tx));
                    if self.mempool.remove_tx(&tx) {
                        self.pending_txs.push(tx);
                    }
                }
                (candidate_ids, candidate_wires)
            }
        };
        

        let tx_root = merkel_root(&candidate_ids);
        let timestamp = unix_time_millis()?;

        let header = BlockHeader {
            parent_hash: parent.hash(),
            height: parent.height + 1,
            tx_root,
            timestamp
        };
        Ok(OrderedBlock::new(header, candidate_wires))
    }

    pub fn clear_pending(&mut self) {
        self.pending_txs.clear();
    }
}

pub fn merkel_root(tx_ids: &[TxAttestationId]) -> Hash32 {
    assert!(!tx_ids.is_empty(), "merkel_root on empty array not defined");
    let mut layer: Vec<Hash32> = tx_ids.iter()
        .map(|tx| tx.0)
        .collect();
    while layer.len() > 1 {
        let mut next = Vec::with_capacity((layer.len() + 1) / 2);
        for pair in layer.chunks(2) {
            let node = if pair.len() == 2 {
                let mut hasher = Sha256::new();
                hasher.update(&pair[0]);
                hasher.update(&pair[1]);
                hasher.finalize().into()
            } else {
                let mut hasher = Sha256::new();
                hasher.update(&pair[0]);
                hasher.finalize().into()
            };
            next.push(node);
        }
        layer = next;
    }
    layer[0]
}