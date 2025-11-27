use thiserror::Error;

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("bcs parse failed")]
    BcsParse(#[from] bcs::Error),
}

pub type Result<T> = std::result::Result<T, LedgerError>;
