use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use account::executor::ExecutorId;
use crate::id::TxId;
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TxServiceCode {
    Success,
    Conflict,
    Timeout,
    InvalidTx,
    FakeReceipt,
    FakeInput,
    NotAssigned,
    BadRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxServiceCatalog(BTreeMap<TxId, (ExecutorId, TxServiceCode)>);


impl TxServiceCatalog {
    pub fn map(&self) -> &BTreeMap<TxId, (ExecutorId, TxServiceCode)> {
        &self.0
    }
    
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self.map()).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(bytes: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(bytes)?)
    }
}
