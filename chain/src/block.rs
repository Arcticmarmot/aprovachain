use std::fmt::{Debug, Formatter};
use serde::{Deserialize, Serialize};
use platform::clock::unix_time_millis;
use primitives::hash::{sha256, Hash32, HASH32_ZERO};
use tx::attestation::{TxAttestation, TxAttestationWire};
use tx::code::{TxServiceCatalog};
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

#[derive(Clone, Serialize, Deserialize)]
pub struct OrderedBlock {
    pub header: BlockHeader,
    pub txs: Vec<TxAttestationWire>
}

impl Debug for OrderedBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // header 直接用派生 Debug
        writeln!(f, "Block {{")?;
        writeln!(f, "  header: {:?},", self.header)?;

        // txs 用你自己的格式
        writeln!(f, "  txs: [")?;
        for tx in &self.txs {
            writeln!(f, "    {:?},", TxAttestation::try_from(tx.clone()).unwrap())?;
        }
        writeln!(f, "  ]")?;
        write!(f, "}}")
    }
}

impl OrderedBlock {
    pub fn new(header: BlockHeader, txs: Vec<TxAttestationWire>) -> Self {
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

#[derive(Clone, Serialize, Deserialize)]
pub struct LedgerBlock {
    pub header: BlockHeader,
    pub txs: Vec<TxAttestationWire>,
    pub tx_codes: TxServiceCatalog
}

impl Debug for LedgerBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Block {{")?;
        writeln!(f, "  header: {:?},", self.header)?;
        writeln!(f, "  txs: [")?;
        for tx in &self.txs {
            writeln!(f, "    {:?},", TxAttestation::try_from(tx.clone()).unwrap())?;
        }
        writeln!(f, "  ]")?;
        for (tx_id, (exec_id, code)) in self.tx_codes.map() {
            writeln!(f, "  tx_id: {tx_id}, exec_id: {exec_id}, code: {code:?}")?;
        }
        write!(f, "}}")
    }
}

impl LedgerBlock {
    pub fn new(header: BlockHeader, txs: Vec<TxAttestationWire>,
               tx_codes: TxServiceCatalog) -> Self {
        Self {
            header,
            txs,
            tx_codes
        }
    }

    pub fn into_ordered_block(self) -> OrderedBlock {
        OrderedBlock {
            header: self.header,
            txs: self.txs
        }
    }

    pub fn try_decode_bcs(bytes: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(bytes)?)
    }

    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }
}