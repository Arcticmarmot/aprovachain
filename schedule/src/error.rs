use thiserror::Error;


#[derive(Debug, Error)]
pub enum ScheduleError {
    #[error(transparent)]
    DB(#[from] db::error::DBError),
    #[error("empty stats to schedule")]
    EmptyStats,
}

pub type Result<T> = std::result::Result<T, ScheduleError>;
