use thiserror::Error;
#[derive(Debug, Error)]
pub enum PeerError {
    #[error("dial peer failed")]
    DialPeer,
    #[error("dial peer failed")]
    SwarmDial(#[from] libp2p::swarm::DialError),
}

pub type Result<T> = std::result::Result<T, PeerError>;
