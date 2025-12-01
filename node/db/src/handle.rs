use std::sync::Arc;
use rocksdb::{ColumnFamily, WriteBatch, DB};
use account::address::{ChainAddrBytes};
use chain::block::{BlockHeader, CommittedBlock};
use contract::contract::{Contract, ContractWire};
use primitives::hash::Hash32;
use apps::ctr_io::{NamespaceKey, ReadSet, ValueSnapshot, WriteSet};
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

    pub fn cf_data(&self) -> &ColumnFamily {
        self.dbh.cf_handle("data").expect(&format!("cf 'data' must be exist"))
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

    pub fn apply_rw_set(&self, read_set: &ReadSet, write_set: &WriteSet) -> Result<bool> {
        // 检查 read_set 视图是否和当前执行过程中一致
        for (ns_key, read_snap_opt) in read_set {
            match (read_snap_opt, self.load_data_entry(&ns_key)?) {
                (Some(read_snap), Some(curr_snap)) => {
                    if read_snap.version != curr_snap.version
                        || read_snap.value != curr_snap.value {
                        return Ok(false);
                    }
                },
                (None, Some(_)) => {
                    return Ok(false)
                },
                (Some(_), None) => {
                    return Ok(false)
                }
                // 读集中没有读到内容，交易仍然成功执行，跳过检查
                (None, None) => { }
            }
        }
        // read_set 检查完毕，开始写入 write_set 到数据库
        for (ns_key, val_opt) in write_set {
            match val_opt {
                Some(val) => {
                    self.save_data_entry(ns_key, val.clone())?;
                },
                None => {
                    self.delete_data_entry(ns_key)?;
                }
            }
        }
        Ok(true)
    }

    pub fn save_data_entry(&self, ns_key: &NamespaceKey, value: Vec<u8>) -> Result<()> {
        let mut batch = WriteBatch::default();
        match self.load_data_entry(ns_key)? {
            Some(snap) => {
                let version = snap.version + 1;
                let new_snap = ValueSnapshot { version, value };
                batch.put_cf(self.cf_data(), ns_key.encode_bcs(), new_snap.encode_bcs());
            },
            None => {
                let new_snap = ValueSnapshot { version: 0, value };
                batch.put_cf(self.cf_data(), ns_key.encode_bcs(), new_snap.encode_bcs());
            }
        }
        self.dbh.write(batch).map_err(DBError::DBPut)?;
        Ok(())
    }

    pub fn delete_data_entry(&self, ns_key: &NamespaceKey) -> Result<()> {
        let mut batch = WriteBatch::default();
        batch.delete_cf(self.cf_data(), ns_key.encode_bcs());
        self.dbh.write(batch).map_err(DBError::DBPut)?;
        Ok(())
    }

    pub fn load_data_entry(&self, ns_key: &NamespaceKey) -> Result<Option<ValueSnapshot>> {
        let data = self.dbh.get_cf(self.cf_data(), ns_key.encode_bcs())
            .map_err(DBError::DBGet)?;
        Ok(match data {
            Some(snap_bytes) => {
                let snap = ValueSnapshot::try_decode_bcs(&snap_bytes)?;
                Some(snap)
            }
            None => None
        })
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
        let ctr_opt = self.dbh.get_cf(self.cf_contracts(), ctr_addr_bytes)
            .map_err(DBError::DBGet)?;
        Ok(match ctr_opt {
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

    pub fn save_block(&self, comm_block: &CommittedBlock) -> Result<()> {
        let mut batch = WriteBatch::default();
        let height_key: [u8; 16] = comm_block.header.height.to_be_bytes();
        batch.put_cf(self.cf_blocks(), height_key, comm_block.encode_bcs());
        self.dbh.write(batch).map_err(DBError::DBPut)?;
        Ok(())
    }

    pub fn load_block(&self, height: u128) -> Result<Option<CommittedBlock>> {
        let height_key: [u8; 16] = height.to_be_bytes();
        let block_opt = self.dbh.get_cf(self.cf_blocks(), height_key)
            .map_err(DBError::DBGet)?;
        Ok(match block_opt{
            Some(block_bytes) => {
                Some(CommittedBlock::try_decode_bcs(&block_bytes)?)
            },
            None => None
        })
    }
}