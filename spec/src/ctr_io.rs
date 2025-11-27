use std::collections::{BTreeSet, HashSet};
use primitives::hash::Hash32;

pub type Version = u128;
#[derive(Debug, Clone)]
pub struct EntryKey {
    pub cf: String,
    pub key: Vec<u8>
}
#[derive(Debug, Clone)]
pub struct ReadEntry {
    pub entry_key: EntryKey,
    pub version: Version,
    pub value: Vec<u8>
}
#[derive(Debug, Clone)]
pub struct WriteEntry {
    pub entry_key: EntryKey,
    pub value: Vec<u8>
}

pub type ReadSet = HashSet<ReadEntry>;
pub type WriteSet = BTreeSet<WriteEntry>;

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