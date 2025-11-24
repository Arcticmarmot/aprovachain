use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use thiserror::Error;
use tx::error::TxError;
use account::error::AccountError;
use contract::error::ContractError;
use db::error::DBError;
use network::error::PeerError;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    Tx(#[from] TxError),
    #[error(transparent)]
    Peer(#[from] PeerError),
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
    #[error("proof generate failed")]
    ProofGenerate(#[source] anyhow::Error),
    #[error("image_id mismatched")]
    ImageIdMismatch,
    #[error("elf_hash mismatched")]
    ElfHashMismatch,
    #[error("elf file not found")]
    ElfFileNotFound,
    #[error("contract not found")]
    ContractNotFound,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::Tx(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            ServerError::DB(e) => (StatusCode::BAD_REQUEST, e.to_string()).into_response(),
            ServerError::ImageIdCompute(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            ServerError::ExecutorEnvBuild(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
            ServerError::ImageIdMismatch => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            ServerError::ElfHashMismatch => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            ServerError::ElfFileNotFound => (StatusCode::BAD_REQUEST, self.to_string()).into_response(),
            _ => (StatusCode::BAD_REQUEST, self.to_string()).into_response()
        }
    }
}

pub type Result<T> = std::result::Result<T, ServerError>;
pub type ApiResult<T> = std::result::Result<Json<T>, ServerError>;