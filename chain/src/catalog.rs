use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use serde::{Deserialize, Serialize};
use account::executor::ExecutorId;
use tx::id::TxId;
use crate::error::Result;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum TxServiceCode {
    Success,
    Conflict,
    Timeout,
    InvalidTx,
    FakeReceipt,
    FakeInput,
    BadRequest,
}

impl Display for TxServiceCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TxServiceCode::Success => { write!(f, "{}", "success") }
            TxServiceCode::Conflict => { write!(f, "{}", "conflict") }
            TxServiceCode::Timeout => { write!(f, "{}", "timeout") }
            _ => { write!(f, "{}", "invalid") }
        }
    }
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
