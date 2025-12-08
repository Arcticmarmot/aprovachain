use serde::{Deserialize, Serialize};
use crate::keypair::AccountVerifyingKey;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorId(pub AccountVerifyingKey);