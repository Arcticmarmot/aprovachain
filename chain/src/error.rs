use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChainError {
    #[error(transparent)]
    Tx(#[from] tx::error::TxError),
    #[error("system time error")]
    SystemTime(#[from] std::time::SystemTimeError),
    #[error("bcs parse failed")]
    BcsParse(#[from] bcs::Error),
}

pub type Result<T> = std::result::Result<T, ChainError>;