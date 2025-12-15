use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use thiserror::Error;
use tx::error::TxError;
use account::error::AccountError;
use apps::error::AppError;
use contract::error::ContractError;
use db::error::DBError;
use engine::error::EngineError;
use network::error::PeerError;
use schedule::error::ScheduleError;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error(transparent)]
    Tx(#[from] TxError),
    #[error(transparent)]
    App(#[from] AppError),
    #[error(transparent)]
    Peer(#[from] PeerError),
    #[error(transparent)]
    Schedule(#[from] ScheduleError),
    #[error(transparent)]
    Account(#[from] AccountError),
    #[error(transparent)]
    DB(#[from] DBError),
    #[error(transparent)]
    Engine(#[from] EngineError),
    #[error(transparent)]
    Contract(#[from] ContractError),
    #[error("risc0 zkvm compute image_id failed")]
    ImageIdCompute(#[source] anyhow::Error),
    #[error("executor env build failed")]
    ExecutorEnvBuild(#[source] anyhow::Error),
    #[error("proof generate failed")]
    ProofGenerate(#[source] anyhow::Error),
    #[error("execute elf failed")]
    ExecuteElf(#[source] anyhow::Error),
    #[error("image_id mismatched")]
    ImageIdMismatch,
    #[error("elf_hash mismatched")]
    ElfHashMismatch,
    #[error("elf file not found")]
    ElfFileNotFound,
    #[error("contract not found")]
    ContractNotFound,
    #[error("receipt not found")]
    ReceiptNotFound,
    #[error("invalid tx scale")]
    InvalidScale,
    #[error("receipt decode failed")]
    ReceiptDecode(#[from] risc0_zkvm::serde::Error),
    #[error("contract exec failed: {message}")]
    ContractExec { message: String },
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        tracing::error!(target: "server::error", error=?self, "api error");
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