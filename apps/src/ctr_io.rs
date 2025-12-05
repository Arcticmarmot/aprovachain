use std::collections::{BTreeMap, BTreeSet};
use serde::{Deserialize, Serialize};
use spec::chain::ChainId;
use primitives::hash::{sha256, Hash32};
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

pub type ReadSet = BTreeMap<NamespaceKey, Option<Vec<u8>>>;
pub type WriteSet = BTreeMap<NamespaceKey, Option<Vec<u8>>>;
pub type AccessSet = BTreeSet<NamespaceKey>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtrInput {
    pub context: CtrContext,
    pub ctx_hash: Hash32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtrContext {
    pub chain_id: ChainId,
    pub input: Vec<u8>,
    pub read_set: ReadSet
}

impl CtrContext {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}

impl CtrInput {
    pub fn create(ctx: &CtrContext) -> Self {
        Self {
            context: ctx.clone(),
            ctx_hash: sha256(ctx.encode_bcs())
        }
    }
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtrOutput {
    pub ctx_hash: Hash32,
    pub read_set: ReadSet,
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
    pub write_set: WriteSet,
    pub answer: Vec<u8>
}

impl CtrOutput {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}
