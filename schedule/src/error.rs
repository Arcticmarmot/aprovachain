use thiserror::Error;


#[derive(Debug, Error)]
pub enum ScheduleError {
    #[error(transparent)]
    DB(#[from] db::error::DBError)
}

pub type Result<T> = std::result::Result<T, ScheduleError>;
