use thiserror::Error;

#[derive(Debug, Error)]
pub enum TxError {
    #[error("OS RNG failed")]
    OsRng(#[from] rand_core::OsError),
    #[error("system time error")]
    SystemTime(#[from] std::time::SystemTimeError),
}