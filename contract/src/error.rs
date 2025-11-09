use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("bcs parse failed")]
    BcsParse(#[from] bcs::Error),
}

pub type Result<T> = std::result::Result<T, ContractError>;