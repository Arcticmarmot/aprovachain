use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChainError {
    #[error("system time error")]
    SystemTime(#[from] std::time::SystemTimeError),
    #[error("bcs parse failed")]
    BcsParse(#[from] bcs::Error),
}

pub type Result<T> = std::result::Result<T, ChainError>;