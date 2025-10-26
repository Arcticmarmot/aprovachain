use std::io;
use thiserror::Error;
use bech32::primitives::hrp::Error as HrpParseError;

#[derive(Debug, Error)]
pub enum AccountError {
    #[error("invalid ed25519 verifying key bytes")]
    InvalidVerifyingKey(#[source] ed25519_dalek::SignatureError),
    #[error("ed25519 signature verification failed")]
    SignatureVerifyingError(#[source] ed25519_dalek::SignatureError),
    #[error("bech32 encode failed")]
    Bech32EncodeError(#[source] bech32::EncodeError),
    #[error("bech32 decode failed")]
    Bech32DecodeError(#[source] bech32::DecodeError),
    #[error("hrp parse failed")]
    HrpParseError(#[from] HrpParseError),
    #[error("hrp mismatched")]
    HrpMismatchError,
    #[error("create dir failed")]
    CreateDirError(#[source] io::Error),
    #[error("write hex failed")]
    WriteHexError(#[source] io::Error),
}