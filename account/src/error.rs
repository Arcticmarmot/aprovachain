use std::io;
use hex::FromHexError;
use thiserror::Error;

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
    #[error("hrp mismatched")]
    HrpMismatch,
    #[error("hrp not in registry")]
    HrpNotInRegistry,
    #[error("create dir failed")]
    CreateDir(#[source] io::Error),
    #[error("write hex failed")]
    WriteHex(#[source] io::Error),
    #[error("decode hex failed")]
    DecodeHex(#[from] FromHexError),
    #[error("decode hex failed")]
    InvalidBytes,
}