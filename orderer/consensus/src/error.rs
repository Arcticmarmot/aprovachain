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
    #[error("config parse failed")]
    ConfigParse(#[from] hex::FromHexError),
    #[error("peer id parse failed")]
    PeerIdParse(#[from] libp2p::identity::ParseError)
}

pub type Result<T> = std::result::Result<T, ConsensusError>;
