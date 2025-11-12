use thiserror::Error;
#[derive(Debug, Error)]
pub enum PeerError {
    // #[error("contract not found")]
    // ContractNotFound,
}

pub type Result<T> = std::result::Result<T, PeerError>;
