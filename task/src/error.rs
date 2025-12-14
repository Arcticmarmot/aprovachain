use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskError {
    #[error(transparent)]
    Engine(#[from] engine::error::EngineError),
    #[error(transparent)]
    Peer(#[from] network::error::PeerError),
    #[error(transparent)]
    Schedule(#[from] schedule::error::ScheduleError),
}

pub type Result<T> = std::result::Result<T, TaskError>;
