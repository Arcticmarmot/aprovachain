use thiserror::Error;

#[derive(Debug, Error)]
pub enum TxError {
    #[error(transparent)]
    Account(#[from] account::error::AccountError),
    #[error("OS RNG failed")]
    OsRng(#[from] rand_core::OsError),
    #[error("system time error")]
    SystemTime(#[from] std::time::SystemTimeError),
    #[error("hex decode error")]
    HexDecode(#[from] hex::FromHexError),
    #[error("tx_intent_id parse error")]
    TxIntentIdPrefix,
    #[error("tx_id parse error")]
    TxIdPrefix,
    #[error("bcs parse failed")]
    BcsParse(#[from] bcs::Error),
    #[error("invalid scale number")]
    TxScaleParse
}

pub type Result<T> = std::result::Result<T, TxError>;