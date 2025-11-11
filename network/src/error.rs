use thiserror::Error;
#[derive(Debug, Error)]
pub enum NodeError {
    #[error("contract not found")]
    ContractNotFound,
}
