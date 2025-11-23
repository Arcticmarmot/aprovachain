use thiserror::Error;
use chain::error::ChainError;
use crate::solo::handle::{SoloCmd, SoloEvent};

#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error(transparent)]
    Chain(#[from] ChainError),
    #[error("send cmd failed")]
    SendSoloCmd(#[from] tokio::sync::mpsc::error::SendError<SoloCmd>),
    #[error("send event failed")]
    SendSoloEvent(#[from] tokio::sync::mpsc::error::SendError<SoloEvent>),
}

pub type Result<T> = std::result::Result<T, ConsensusError>;
