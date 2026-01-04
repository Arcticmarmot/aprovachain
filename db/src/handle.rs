use std::sync::Arc;
use rocksdb::{ColumnFamily, WriteBatch, DB};
use account::address::{ChainAddrBytes};
use chain::block::{BlockHeader, LedgerBlock, OrderedBlock};
use contract::contract::{Contract, ContractWire};
use primitives::hash::Hash32;
use apps::ctr_io::{NamespaceKey, ReadSet, WriteSet};
use chain::catalog::TxServiceCatalog;
use platform::config::load_base_config;
use crate::error::DBError;
use crate::runtime::dbh;
use crate::error::Result;

pub const CHAIN_TIP_KEY: &[u8] = b"tip_header";
pub const GENESIS_TS_KEY: &[u8] = b"genesis_ts";

#[derive(Debug, Clone)]
pub struct DBHandle {
    pub dbh: Arc<DB>,
    pub slot_secs: u64
}

impl DBHandle {
    pub fn new() -> Result<Self> {
        let dbh = dbh()?;
        let base = load_base_config();
        let slot_secs = base.slot_secs;
        Ok(Self { dbh, slot_secs })
    }

    pub fn cf_data(&self) -> &ColumnFamily {
        self.dbh.cf_handle("data").expect("cf 'data' must be exist")
    }
    
    pub fn cf_chain(&self) -> &ColumnFamily {
        self.dbh.cf_handle("chain").expect("cf 'chain' must be exist")
    }

    pub fn cf_blocks(&self) -> &ColumnFamily {
        self.dbh.cf_handle("blocks").expect("cf 'blocks' must be exist")
    }

    pub fn cf_catalog(&self) -> &ColumnFamily {
        self.dbh.cf_handle("catalog").expect("cf 'catalog' must be exist")
    }

    pub fn cf_contracts(&self) -> &ColumnFamily {
        self.dbh.cf_handle("contracts").expect("cf 'contracts' must be exist")
    }

    pub fn cf_elfs(&self) -> &ColumnFamily {
        self.dbh.cf_handle("elfs").expect("cf 'elfs' must be exist")
    }
    
    pub fn load_ts_height(&self, ts: u128) -> Result<Option<u128>> {
        Ok(match self.load_genesis_ts()? {
            Some(genesis_ts) => {
                // Note: 共识服务稳定出块才能一致
                let now_slot = ts.saturating_sub(genesis_ts) / (self.slot_secs * 1000) as u128;
                Some(now_slot)
            }
            None => { None }
        })
    }

    pub fn load_stats_window(&self, window_size: usize, ts: u128) -> Result<Vec<TxServiceCatalog>> {
        match self.load_ts_height(ts)? {
            Some(end_height) => {
                let start_height= end_height.saturating_sub(window_size as u128);
                tracing::info!(target: "db::window", %start_height, %end_height, "window");
                let mut stats = Vec::with_capacity(window_size);
                for height in start_height..end_height {
                    match self.load_catalog(height)? {
                        Some(catalog) => {
                            stats.push(catalog)
                        },
                        None => { return Err(DBError::DBIntegrity) }
                    }
                }
                Ok(stats)
            }
            None => { Ok(Vec::new()) }
        }
    }

    pub fn apply_rw_set(&self, read_set: &ReadSet, write_set: &WriteSet) -> Result<bool> {
        // 检查 read_set 视图是否和当前执行过程中一致
        for (ns_key, read_value_opt) in read_set {
            match (read_value_opt, self.load_data_entry(&ns_key)?) {
                (Some(read_value), Some(curr_value)) => {
                    if read_value != &curr_value {
                        return Ok(true);
                    }
                },
                (None, Some(_)) => {
                    return Ok(true)
                },
                (Some(_), None) => {
                    return Ok(true)
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
        Ok(false)
    }

    pub fn save_data_entry(&self, ns_key: &NamespaceKey, value: Vec<u8>) -> Result<()> {
        let mut batch = WriteBatch::default();
        batch.put_cf(self.cf_data(), ns_key.encode_bcs(), value);
        self.dbh.write(batch).map_err(DBError::DBPut)?;
        Ok(())
    }

    pub fn delete_data_entry(&self, ns_key: &NamespaceKey) -> Result<()> {
        let mut batch = WriteBatch::default();
        batch.delete_cf(self.cf_data(), ns_key.encode_bcs());
        self.dbh.write(batch).map_err(DBError::DBPut)?;
        Ok(())
    }

    pub fn load_data_entry(&self, ns_key: &NamespaceKey) -> Result<Option<Vec<u8>>> {
        Ok(self.dbh.get_cf(self.cf_data(), ns_key.encode_bcs()).map_err(DBError::DBGet)?)
    }

    pub fn save_chain_state(&self, header: &BlockHeader) -> Result<()> {
        let mut batch = WriteBatch::default();
        if header.height == 0 {
            batch.put_cf(self.cf_chain(), GENESIS_TS_KEY, header.timestamp.to_be_bytes());
        }
        batch.put_cf(self.cf_chain(), CHAIN_TIP_KEY, header.encode_bcs());
        self.dbh.write(batch).map_err(DBError::DBPut)?;
        Ok(())
    }

    pub fn load_genesis_ts(&self) -> Result<Option<u128>> {
        let genesis_ts_opt = self.dbh.get_cf(self.cf_chain(), GENESIS_TS_KEY)
            .map_err(DBError::DBGet)?;
        Ok(match genesis_ts_opt {
            Some(ts_bytes) => {
                let mut out = [0u8; 16];
                out.copy_from_slice(&ts_bytes);
                Some(u128::from_be_bytes(out))
            },
            None => None
        })
    }
    
    pub fn load_chain_state(&self) -> Result<Option<BlockHeader>> {
        let tip_header_opt = self.dbh.get_cf(self.cf_chain(), CHAIN_TIP_KEY)
            .map_err(DBError::DBGet)?;
        Ok(match tip_header_opt {
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

    pub fn save_block(&self, ledger_block: &LedgerBlock) -> Result<()> {
        let mut batch = WriteBatch::default();
        let ordered_block = &ledger_block.ordered;
        let height_key: [u8; 16] = ordered_block.header.height.to_be_bytes();
        let catalog = &ledger_block.catalog;
        batch.put_cf(self.cf_blocks(), height_key, ordered_block.encode_bcs());
        batch.put_cf(self.cf_catalog(), height_key, catalog.encode_bcs());
        self.dbh.write(batch).map_err(DBError::DBPut)?;
        Ok(())
    }

    pub fn load_catalog(&self, height: u128) -> Result<Option<TxServiceCatalog>> {
        let height_key: [u8; 16] = height.to_be_bytes();
        let catalog_opt = self.dbh.get_cf(self.cf_catalog(), height_key)
            .map_err(DBError::DBGet)?;
        Ok(match catalog_opt {
            Some(catalog_bytes) => {
                Some(TxServiceCatalog::try_decode_bcs(&catalog_bytes)?)
            }
            None => None
        })
    }

    pub fn load_block(&self, height: u128) -> Result<Option<LedgerBlock>> {
        let height_key: [u8; 16] = height.to_be_bytes();
        let ordered_block_opt = self.dbh.get_cf(self.cf_blocks(), height_key)
            .map_err(DBError::DBGet)?;
        let catalog_opt = self.load_catalog(height)?;
        Ok(match (ordered_block_opt, catalog_opt){
            (Some(block_bytes), Some(catalog)) => {
                let ordered_block = OrderedBlock::try_decode_bcs(&block_bytes)?;
                Some(LedgerBlock::new(ordered_block, catalog))
            },
            _ => None
        })
    }
}