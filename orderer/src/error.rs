use thiserror::Error;
use tx::error::TxError;
use account::error::AccountError;
use contract::error::ContractError;

#[derive(Debug, Error)]
pub enum OrdererError {
    #[error(transparent)]
    Tx(#[from] TxError),
    #[error(transparent)]
    Account(#[from] AccountError),
    #[error(transparent)]
    Contract(#[from] ContractError),
}

pub type Result<T> = std::result::Result<T, OrdererError>;