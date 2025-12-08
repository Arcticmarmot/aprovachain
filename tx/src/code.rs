use serde::{Deserialize, Serialize};

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