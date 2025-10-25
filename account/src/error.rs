use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccountError {
    #[error("invalid ed25519 verifying key bytes")]
    InvalidVerifyingKey(#[source] ed25519_dalek::SignatureError),
    #[error("ed25519 signature verification failed")]
    SignatureVerifyingError(#[source] ed25519_dalek::SignatureError),
}