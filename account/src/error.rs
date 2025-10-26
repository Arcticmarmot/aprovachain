use std::io;
use thiserror::Error;
use bech32::primitives::hrp::Error as HrpParseError;

#[derive(Debug, Error)]
pub enum AccountError {
    #[error("invalid ed25519 verifying key bytes")]
    InvalidVerifyingKey(#[source] ed25519_dalek::SignatureError),
    #[error("ed25519 signature verification failed")]
    SignatureVerify(#[source] ed25519_dalek::SignatureError),
    #[error("bech32 encode failed")]
    Bech32Encode(#[source] bech32::EncodeError),
    #[error("bech32 decode failed")]
    Bech32Decode(#[source] bech32::DecodeError),
    #[error("hrp parse failed")]
    HrpParse(#[from] HrpParseError),
    #[error("hrp mismatched")]
    HrpMismatch,
    #[error("create dir failed")]
    CreateDir(#[source] io::Error),
    #[error("write hex failed")]
    WriteHex(#[source] io::Error),
}