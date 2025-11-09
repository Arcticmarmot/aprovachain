use thiserror::Error;

#[derive(Debug, Error)]
pub enum ChainError {

}

pub type Result<T> = std::result::Result<T, ChainError>;