use std::collections::{BTreeSet};
use serde::{Deserialize, Serialize};
use spec::error::Result;

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

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct ReadEntry {
    pub key: NamespaceKey,
    pub snap: ValueSnapShot
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct WriteEntry {
    pub key: NamespaceKey,
    pub snap: ValueSnapShot
}

pub type ReadSet = BTreeSet<ReadEntry>;
pub type WriteSet = BTreeSet<WriteEntry>;
pub type AccessSet = BTreeSet<NamespaceKey>;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtrInput {
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
    pub read_set: ReadSet,
    pub write_set: WriteSet,
    pub output: Vec<u8>
}

impl CtrOutput {
    pub fn encode_bcs(&self) -> Vec<u8> {
        bcs::to_bytes(self).expect("BCS should be infallible by design")
    }

    pub fn try_decode_bcs(b: &[u8]) -> Result<Self> {
        Ok(bcs::from_bytes(b)?)
    }
}
