use crate::error::DBError;
use crate::init::open_db;

type Result<T> = std::result::Result<T, DBError>;

pub fn kv_put(key: &[u8], value: &[u8]) -> Result<()> {
    let db = open_db()?;
    db.put(key, value).map_err(DBError::DBPut)?;
    Ok(())
}

pub fn kv_get(key: &[u8]) -> Result<Option<Vec<u8>>> {
    let db = open_db()?;
    let value = db.get(key).map_err(DBError::DBOpen)?;
    Ok(value)
}