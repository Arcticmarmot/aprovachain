use thiserror::Error;
use tx::error::TxError;
use account::error::AccountError;
use contract::error::ContractError;
use db::error::DBError;
use network::error::PeerError;

#[derive(Debug, Error)]
pub enum NodeError {
    #[error(transparent)]
    Tx(#[from] TxError),
    #[error(transparent)]
    Peer(#[from] PeerError),
    #[error(transparent)]
    Account(#[from] AccountError),
    #[error(transparent)]
    DB(#[from] DBError),
    #[error(transparent)]
    Contract(#[from] ContractError),
    #[error("risc0 zkvm compute image_id failed")]
    ImageIdCompute(#[source] anyhow::Error),
    #[error("executor env build failed")]
    ExecutorEnvBuild(#[source] anyhow::Error),
    #[error("proof generate failed")]
    ProofGenerate(#[source] anyhow::Error),
    #[error("image_id mismatched")]
    ImageIdMismatch,
    #[error("elf_hash mismatched")]
    ElfHashMismatch,
    #[error("elf file not found")]
    ElfFileNotFound,
    #[error("contract not found")]
    ContractNotFound,
}