use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use account::executor::ExecutorId;
use crate::id::TxId;

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

pub type TxServiceCodeMap = BTreeMap<TxId, (ExecutorId, TxServiceCode)>;
