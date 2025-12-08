use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use primitives::hash::Hash32;
use crate::error::TxError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Intent {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Envelope {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Outcome {}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Attestation {}

pub type TxIntentId = TxCommonId<Intent>;
pub type TxEnvelopeId = TxCommonId<Envelope>;
pub type TxOutcomeId = TxCommonId<Outcome>;
pub type TxAttestationId = TxCommonId<Attestation>;
pub type TxId = TxAttestationId;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TxCommonId<T>(pub Hash32, PhantomData<T>);


impl<T> Debug for TxCommonId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
impl<T> Display for TxCommonId<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x{}", hex::encode(&self.0))
    }
}

impl<T> FromStr for TxCommonId<T> {
    type Err = TxError;

    fn from_str(s: &str) -> crate::error::Result<Self> {
        let s = s.strip_prefix("0x").ok_or(TxError::TxIntentIdPrefix)?;
        let mut buf: Hash32 = [0u8; 32];
        hex::decode_to_slice(s, &mut buf)?;
        Ok(TxCommonId(buf, PhantomData))
    }
}

impl<T> TxCommonId<T> {
    pub fn new(hash: Hash32) -> Self {
        Self(hash, PhantomData)
    }
}




