use thiserror::Error;
use tx::error::TxError;
use account::error::AccountError;
use consensus::error::ConsensusError;
use contract::error::ContractError;
use network::error::PeerError;

#[derive(Debug, Error)]
pub enum OrdererError {
    #[error(transparent)]
    Tx(#[from] TxError),
    #[error(transparent)]
    Account(#[from] AccountError),
    #[error(transparent)]
    Contract(#[from] ContractError),
    #[error(transparent)]
    Consensus(#[from] ConsensusError),
    #[error(transparent)]
    Peer(#[from] PeerError)
}

pub type Result<T> = std::result::Result<T, OrdererError>;