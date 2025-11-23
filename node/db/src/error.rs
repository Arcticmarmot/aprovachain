use thiserror::Error;
use rocksdb::Error as RocksDBError;
use chain::error::ChainError;
use contract::error::ContractError;

#[derive(Debug, Error)]
pub enum DBError {
    #[error(transparent)]
    Chain(#[from] ChainError),
    #[error(transparent)]
    Contract(#[from] ContractError),
    #[error("DB not init")]
    DBNotInit,
    #[error("fixed dir create failed")]
    DBFixedDirCreate,
    #[error("DB dir create failed")]
    DBTempDirCreate(#[source] std::io::Error),
    #[error("DB open failed")]
    DBOpen(#[source] RocksDBError),
    #[error("DB put failed")]
    DBPut(#[source] RocksDBError),
    #[error("DB get failed")]
    DBGet(#[source] RocksDBError),
}

pub type Result<T> = std::result::Result<T, DBError>;
