use thiserror::Error;
use bech32::primitives::hrp::Error as HrpParseError;

#[derive(Debug, Error)]
pub enum SpecError {
    #[error("hrp parse failed")]
    HrpParse(#[from] HrpParseError),
}

pub type Result<T> = std::result::Result<T, SpecError>;
