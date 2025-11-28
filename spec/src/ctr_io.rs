use std::collections::{BTreeSet};
use serde::{Deserialize, Serialize};
use primitives::hash::Hash32;
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
pub struct ValueSnapShot {
    pub version: u128,
    pub value: Option<Vec<u8>>
}

impl ValueSnapShot {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}

#[derive(Debug, Clone)]
pub struct ReadEntry {
    pub key: NamespaceKey,
    pub snap: ValueSnapShot
}

#[derive(Debug, Clone)]
pub struct WriteEntry {
    pub key: NamespaceKey,
    pub snap: ValueSnapShot
}

pub type ReadSet = BTreeSet<ReadEntry>;
pub type WriteSet = BTreeSet<WriteEntry>;
pub type AccessSet = BTreeSet<NamespaceKey>;


#[derive(Debug, Clone)]
struct CtrInput {
    input: Vec<u8>,
    context: EnvContext
}

#[derive(Debug, Clone)]
struct EnvContext {
    read_set: ReadSet
}

#[derive(Debug, Clone)]
struct CtrOutput {
    input_hash: Hash32,
    read_set: ReadSet,
    write_set: WriteSet,
    output: Vec<u8>
}