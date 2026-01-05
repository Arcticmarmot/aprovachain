use std::collections::BTreeMap;
use std::ops::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use account::executor::ExecutorId;
use tx::id::TxId;
use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TxServiceCode {
    Success,
    Conflict,
    Timeout,
    ValidTx,
    InvalidTx,
    FakeReceipt,
    FakeInput,
    NotAssigned,
    BadRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxServiceCatalog(BTreeMap<TxId, (ExecutorId, TxServiceCode)>);

impl Deref for TxServiceCatalog {
    type Target = BTreeMap<TxId, (ExecutorId, TxServiceCode)>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TxServiceCatalog {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TxServiceCatalog {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(bytes: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(bytes)?)
    }
}
