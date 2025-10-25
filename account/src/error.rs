use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccountError {
    #[error("invalid ed25519 verifying key bytes")]
    InvalidVerifyingKey(#[source] ed25519_dalek::SignatureError),
    #[error("ed25519 signature verification failed")]
    SignatureVerifyingError(#[source] ed25519_dalek::SignatureError),
    #[error("bech32 encode failed")]
    Bech32EncodeError(#[source] bech32::EncodeError)
}