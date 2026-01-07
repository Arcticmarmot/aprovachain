use thiserror::Error;
use chain::error::ChainError;
use crate::cft::protocol::{CftCmd, CftEvent};
use crate::solo::protocol::{SoloCmd, SoloEvent};

#[derive(Debug, Error)]
pub enum ConsensusError {
    #[error(transparent)]
    Chain(#[from] ChainError),
    #[error("send cmd failed")]
    SendSoloCmd(#[from] tokio::sync::mpsc::error::SendError<SoloCmd>),
    #[error("send event failed")]
    SendSoloEvent(#[from] tokio::sync::mpsc::error::SendError<SoloEvent>),
    #[error("send cmd failed")]
    SendCftCmd(#[from] tokio::sync::mpsc::error::SendError<CftCmd>),
    #[error("send event failed")]
    SendCftEvent(#[from] tokio::sync::mpsc::error::SendError<CftEvent>),
    #[error("bcs parse failed")]
    BcsParse(#[from] bcs::Error),
    #[error("bad consensus mode")]
    ConsensusMode
}

pub type Result<T> = std::result::Result<T, ConsensusError>;
