use thiserror::Error;
use tx::error::TxError;
use crate::handle::{P2pCmd, TxCmd};

#[derive(Debug, Error)]
pub enum PeerError {
    #[error(transparent)]
    Tx(#[from] TxError),
    #[error("dial peer failed")]
    DialPeer,
    #[error("dial peer failed")]
    SwarmDial(#[from] libp2p::swarm::DialError),
    #[error("send cmd failed")]
    SendP2pCmd(#[from] tokio::sync::mpsc::error::SendError<P2pCmd>),
    #[error("send cmd failed")]
    SendTxCmd(#[from] tokio::sync::mpsc::error::SendError<TxCmd>),
    #[error("gossip publish failed")]
    GossipPublish(#[from] libp2p::gossipsub::PublishError)
}

pub type Result<T> = std::result::Result<T, PeerError>;
