use thiserror::Error;
use chain::error::ChainError;

#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error(transparent)]
    Chain(#[from] ChainError),
}

pub type Result<T> = std::result::Result<T, ConsensusError>;
