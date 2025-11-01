use thiserror::Error;
use rocksdb::Error as RocksDBError;
#[derive(Debug, Error)]
pub enum DBError {
    #[error("DB dir init failed")]
    DBDirInit,
    #[error("DB open failed")]
    DBOpen(#[source] RocksDBError),
    #[error("DB put failed")]
    DBPut(#[source] RocksDBError),
    #[error("DB get failed")]
    DBGet(#[source] RocksDBError),
}

pub type Result<T> = std::result::Result<T, DBError>;
