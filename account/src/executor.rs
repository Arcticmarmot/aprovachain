use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};
use crate::keypair::AccountVerifyingKey;


#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutorId(pub AccountVerifyingKey);

impl Hash for ExecutorId {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write(&self.0.to_bytes());
    }
}