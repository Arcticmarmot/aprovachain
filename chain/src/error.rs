use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChainError {
    #[error("system time error")]
    SystemTime(#[from] std::time::SystemTimeError),
}

pub type Result<T> = std::result::Result<T, ChainError>;