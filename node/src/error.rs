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
    DB(#[from] DBError)
}

impl IntoResponse for NodeError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}

pub type Result<T> = std::result::Result<T, NodeError>;