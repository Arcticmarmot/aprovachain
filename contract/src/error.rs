use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContractError {
    
}

pub type Result<T> = std::result::Result<T, ContractError>;