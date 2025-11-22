use thiserror::Error;
use crate::handle::{P2pCmd, P2pEvent};

#[derive(Debug, Error)]
pub enum PeerError {
    #[error("dial peer failed")]
    DialPeer,
    #[error("dial peer failed")]
    SwarmDial(#[from] libp2p::swarm::DialError),
    #[error("send cmd failed")]
    SendP2pCmd(#[from] tokio::sync::mpsc::error::SendError<P2pCmd>),
    #[error("send event failed")]
    SendTxCmd(#[from] tokio::sync::mpsc::error::SendError<P2pEvent>),
    #[error("gossip publish failed")]
    GossipPublish(#[from] libp2p::gossipsub::PublishError)
}

pub type Result<T> = std::result::Result<T, PeerError>;
