use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use thiserror::Error;
use tx::error::TxError;
use account::error::AccountError;
use contract::error::ContractError;
use db::error::DBError;

#[derive(Debug, Error)]
pub enum NodeError {
    #[error(transparent)]
    Tx(#[from] TxError),
    #[error(transparent)]
    Account(#[from] AccountError),
    #[error(transparent)]
    DB(#[from] DBError),
    #[error(transparent)]
    Contract(#[from] ContractError),
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
        match self {
            NodeError::Tx(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            NodeError::DB(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            NodeError::ImageIdCompute(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            NodeError::ExecutorEnvBuild(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            NodeError::ImageIdMismatch => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            NodeError::ElfHashMismatch => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            NodeError::ElfFileNotFound => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            _ => (StatusCode::BAD_REQUEST, self.to_string()).into_response()
        }
    }
}

pub type Result<T> = std::result::Result<T, NodeError>;
pub type ApiResult<T> = std::result::Result<Json<T>, NodeError>;