use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccountError {
    #[error("bech32 error: {0}")]
    Bech32(#[from] bech32::Error),
    #[error("ed25519 signature error")]
    Signature
}