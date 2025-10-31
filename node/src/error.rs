use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;
use tx::error::TxError;

#[derive(Debug, Error)]
pub enum NodeError {
    #[error(transparent)]
    Tx(#[from] TxError)
}

impl IntoResponse for NodeError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}