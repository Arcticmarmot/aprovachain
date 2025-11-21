use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use primitives::hash::Hash32;
use crate::error::TxError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Intent {}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Envelope {}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Exec {}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExecSeal {}

pub type TxIntentId = TxId<Intent>;
pub type TxEnvelopeId = TxId<Envelope>;
pub type TxExecId = TxId<Exec>;
pub type TxExecSealId = TxId<ExecSeal>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TxId<T>(pub Hash32, PhantomData<T>);

impl<T> Display for TxId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl<T> FromStr for TxId<T> {
    type Err = TxError;

    fn from_str(s: &str) -> crate::error::Result<Self> {
        let s = s.strip_prefix("0x").ok_or(TxError::TxIntentIdPrefix)?;
        let mut buf: Hash32 = [0u8; 32];
        hex::decode_to_slice(s, &mut buf)?;
        Ok(TxId(buf, PhantomData))
    }
}

impl<T> TxId<T> {
    pub fn new(hash: Hash32) -> Self {
        Self(hash, PhantomData)
    }
}




