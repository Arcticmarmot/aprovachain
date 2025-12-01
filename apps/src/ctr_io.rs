use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use spec::chain::ChainId;
use crate::error::Result;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct NamespaceKey {
    pub ns: String,
    pub key: Vec<u8>
}

impl NamespaceKey {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct ValueSnapshot {
    pub version: u128,
    pub value: Vec<u8>
}

impl ValueSnapshot {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}

pub type ReadSet = BTreeMap<NamespaceKey, Option<ValueSnapshot>>;
pub type WriteSet = BTreeMap<NamespaceKey, Option<ValueSnapshot>>;
pub type AccessSet = BTreeSet<NamespaceKey>;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtrInput {
    pub chain_id: ChainId,
    pub input: Vec<u8>,
    pub context: EnvContext
}

impl CtrInput {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvContext {
    pub read_set: ReadSet
}

impl EnvContext {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtrOutput {
    pub input_hash: [u8; 32],
    pub ctr_result: CtrResult
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CtrResult {
    Ok {
        outcome: CtrOutcome
    },
    Err {
        message: String
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtrOutcome {
    pub effects: CtrEffects,
    pub answer: Vec<u8>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtrEffects {
    pub read_set: ReadSet,
    pub write_set: WriteSet
}

impl CtrOutput {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}
