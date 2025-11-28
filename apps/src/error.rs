use thiserror::Error;
use bech32::primitives::hrp::Error as HrpParseError;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("hrp parse failed")]
    HrpParse(#[from] HrpParseError),
    #[error("bcs parse failed")]
    BcsParse(#[from] bcs::Error),
}

pub type Result<T> = std::result::Result<T, AppError>;
