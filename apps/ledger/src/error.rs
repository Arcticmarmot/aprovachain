use thiserror::Error;
use account::error::AccountError;

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error(transparent)]
    Account(#[from] AccountError),
    #[error("bcs parse failed")]
    BcsParse(#[from] bcs::Error),
}

pub type Result<T> = std::result::Result<T, LedgerError>;
