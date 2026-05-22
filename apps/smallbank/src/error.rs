use thiserror::Error;
use account::error::AccountError;

#[derive(Debug, Error)]
pub enum SmallbankError {
    #[error(transparent)]
    Account(#[from] AccountError),
    #[error("bcs parse failed")]
    BcsParse(#[from] bcs::Error),
}

pub type Result<T> = std::result::Result<T, SmallbankError>;