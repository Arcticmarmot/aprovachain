use std::sync::Arc;
use rocksdb::{ColumnFamily, WriteBatch, DB};
use account::address::{ChainAddrBytes};
use chain::block::BlockHeader;
use contract::contract::{Contract, ContractWire};
use primitives::hash::Hash32;
use crate::error::DBError;
use crate::runtime::dbh;
use crate::error::Result;

pub const CHAIN_TIP_KEY: &[u8] = b"tip_header";

#[derive(Debug, Clone)]
pub struct DBHandle {
    pub dbh: Arc<DB>,
}

impl DBHandle {
    pub fn new() -> Result<Self> {
        let dbh = dbh()?;
        Ok(Self { dbh })
    }

    pub fn cf_chain(&self) -> &ColumnFamily {
        self.dbh.cf_handle("chain").expect("cf 'chain' must be exist")
    }

    pub fn cf_blocks(&self) -> &ColumnFamily {
        self.dbh.cf_handle("blocks").expect("cf 'blocks' must be exist")
    }

    pub fn cf_contracts(&self) -> &ColumnFamily {
        self.dbh.cf_handle("contracts").expect("cf 'contracts' must be exist")
    }

    pub fn cf_elfs(&self) -> &ColumnFamily {
        self.dbh.cf_handle("elfs").expect("cf 'elfs' must be exist")
    }

    pub fn save_chain_state(&self, header: &BlockHeader) -> Result<()> {
        let mut batch = WriteBatch::default();
        batch.put_cf(self.cf_chain(), CHAIN_TIP_KEY, header.encode_bcs());
        self.dbh.write(batch).map_err(DBError::DBPut)?;
        Ok(())
    }

    pub fn load_chain_state(&self) -> Result<Option<BlockHeader>> {
        let tip_header = self.dbh.get_cf(self.cf_chain(), CHAIN_TIP_KEY)
            .map_err(DBError::DBGet)?;
        Ok(match tip_header {
            Some(header_bytes) => { Some(BlockHeader::try_decode_bcs(&header_bytes)?) },
            None => None
        })
    }

    pub fn save_contract(&self, ctr_addr_bytes: &ChainAddrBytes, ctr_bytes: &[u8]) -> Result<()> {
        let mut batch = WriteBatch::default();
        batch.put_cf(self.cf_contracts(), ctr_addr_bytes, ctr_bytes);
        self.dbh.write(batch).map_err(DBError::DBPut)?;
        Ok(())
    }

    pub fn load_contract(&self, ctr_addr_bytes: &ChainAddrBytes) -> Result<Option<Contract>> {
        let ctr = self.dbh.get_cf(self.cf_contracts(), ctr_addr_bytes)
            .map_err(DBError::DBGet)?;
        Ok(match ctr {
            Some(ctr_bytes) => {
                let wire = ContractWire::try_encode_bcs(&ctr_bytes)?;
                Some(Contract::try_from(wire)?)
            },
            None => None
        })
    }

    pub fn save_elf(&self, elf_hash: Hash32, elf_bytes: &[u8]) -> Result<()> {
        let mut batch = WriteBatch::default();
        batch.put_cf(self.cf_elfs(), elf_hash, elf_bytes);
        self.dbh.write(batch).map_err(DBError::DBPut)?;
        Ok(())
    }

    pub fn load_elf(&self, elf_hash: Hash32) -> Result<Option<Vec<u8>>> {
        let elf = self.dbh.get_cf(self.cf_elfs(), elf_hash)
            .map_err(DBError::DBGet)?;
        Ok(elf)
    }
}