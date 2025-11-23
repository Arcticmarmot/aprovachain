use rocksdb::WriteBatch;
use chain::block::BlockHeader;
use crate::error::{Result, DBError};
use crate::runtime::{dbh};

pub fn kv_put(key: &[u8], value: &[u8]) -> Result<()> {
    let dbh = dbh()?;
    dbh.put(key, value).map_err(DBError::DBPut)?;
    Ok(())
}

pub fn kv_get(key: &[u8]) -> Result<Option<Vec<u8>>> {
    let dbh = dbh()?;
    let value = dbh.get(key).map_err(DBError::DBGet)?;
    Ok(value)
}

pub fn load_tip_chain_state() -> Result<()> {
    let dbh = dbh()?;
    let cf_chain = dbh.cf_handle("chain").expect("chain must exist");
    dbh.get_cf(cf_chain, b"tip").map_err(DBError::DBGet)?;
    Ok(())
}

pub fn update_chain_state(header: &BlockHeader) -> Result<()>{
    let dbh = dbh()?;
    let mut batch = WriteBatch::default();
    let cf_chain = dbh.cf_handle("chain").expect("chain must exist");
    batch.put_cf(cf_chain, b"tip", header.to_canonical_bytes());
    dbh.write(batch).map_err(DBError::DBPut)?;
    Ok(())
}