use thiserror::Error;
use tx::error::TxError;

#[derive(Debug, Error)]
pub enum PeerError {
    #[error(transparent)]
    Tx(#[from] TxError),
    #[error("dial peer failed")]
    DialPeer,
    #[error("dial peer failed")]
    SwarmDial(#[from] libp2p::swarm::DialError),
}

pub type Result<T> = std::result::Result<T, PeerError>;
