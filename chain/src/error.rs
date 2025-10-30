use thiserror::Error;
use bech32::primitives::hrp::Error as HrpParseError;

#[derive(Debug, Error)]
pub enum ChainError {
    #[error("hrp parse failed")]
    HrpParse(#[from] HrpParseError),
}