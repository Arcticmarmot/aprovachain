use crate::error::{Result, DBError};
use crate::init::DBH;

pub fn kv_put(key: &[u8], value: &[u8]) -> Result<()> {
    let db = DBH.get().unwrap().load_full().unwrap();
    db.put(key, value).map_err(DBError::DBPut)?;
    Ok(())
}

pub fn kv_get(key: &[u8]) -> Result<Option<Vec<u8>>> {
    let db = DBH.get().unwrap().load_full().unwrap();
    let value = db.get(key).map_err(DBError::DBOpen)?;
    Ok(value)
}