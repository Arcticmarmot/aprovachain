use thiserror::Error;
use tx::error::TxError;
use account::error::AccountError;
use apps::error::AppError;
use contract::error::ContractError;
use db::error::DBError;
use network::error::PeerError;
use schedule::error::ScheduleError;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error(transparent)]
    Tx(#[from] TxError),
    #[error(transparent)]
    App(#[from] AppError),
    #[error(transparent)]
    Peer(#[from] PeerError),
    #[error(transparent)]
    Schedule(#[from] ScheduleError),
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
    #[error("execute elf failed")]
    ExecuteElf(#[source] anyhow::Error),
    #[error("image_id mismatched")]
    ImageIdMismatch,
    #[error("elf_hash mismatched")]
    ElfHashMismatch,
    #[error("elf file not found")]
    ElfFileNotFound,
    #[error("contract not found")]
    ContractNotFound,
    #[error("receipt not found")]
    ReceiptNotFound,
    #[error("invalid tx scale")]
    InvalidScale,
    #[error("receipt decode failed")]
    ReceiptDecode(#[from] risc0_zkvm::serde::Error),
    #[error("contract exec failed: {message}")]
    ContractExec { message: String },
    #[error("bad prove scheme")]
    ProveScheme,
    #[error("receipt file not found")]
    ReceiptFileNotFound(#[from] anyhow::Error)
}

pub type Result<T> = std::result::Result<T, EngineError>;