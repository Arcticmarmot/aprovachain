use std::collections::{BTreeSet};
use serde::{Deserialize, Serialize};
use primitives::hash::Hash32;

pub type Version = u128;
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct EntryKey {
    pub cf: String,
    pub key: Vec<u8>
}
#[derive(Debug, Clone)]
pub struct ReadEntry {
    pub entry_key: EntryKey,
    pub version: Version,
    pub value: Option<Vec<u8>>
}
#[derive(Debug, Clone)]
pub struct WriteEntry {
    pub entry_key: EntryKey,
    pub value: Vec<u8>
}

pub type ReadSet = BTreeSet<ReadEntry>;
pub type WriteSet = BTreeSet<WriteEntry>;
pub type AccessSet = BTreeSet<EntryKey>;


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