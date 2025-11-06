use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;
use tx::error::TxError;
use account::error::AccountError;
use db::error::DBError;

#[derive(Debug, Error)]
pub enum NodeError {
    #[error(transparent)]
    Tx(#[from] TxError),
    #[error(transparent)]
    Account(#[from] AccountError),
    #[error(transparent)]
    DB(#[from] DBError),
    #[error("risc0 zkvm compute image_id failed")]
    ImageIdCompute(#[source] anyhow::Error),
    #[error("executor env build failed")]
    ExecutorEnvBuild(#[source] anyhow::Error),
    #[error("image_id mismatched")]
    ImageIdMismatch,
    #[error("elf_hash mismatched")]
    ElfHashMismatch,
    #[error("elf file not found")]
    ElfFileNotFound,
}

impl IntoResponse for NodeError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}

pub type Result<T> = std::result::Result<T, NodeError>;