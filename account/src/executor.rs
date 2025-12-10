use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash};
use serde::{Deserialize, Serialize};
use crate::keypair::AccountVerifyingKey;

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ExecutorId(pub AccountVerifyingKey);

impl PartialOrd for ExecutorId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ExecutorId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.verifying_key().to_bytes().cmp(&other.verifying_key().to_bytes())
    }
}

impl Display for ExecutorId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.verifying_key().to_bytes()))
    }
}

impl Debug for ExecutorId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.verifying_key().to_bytes()))
    }
}

impl ExecutorId {
    pub fn verifying_key(&self) -> AccountVerifyingKey {
        self.0
    }
}