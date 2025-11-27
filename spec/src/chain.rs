use std::fmt::{Display, Formatter};
use bech32::Hrp;
use serde::{Deserialize, Serialize};
use crate::error::Result;

pub static CTR_PREFIX: &'static str = "ctr";

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChainId(pub u64);

impl Display for ChainId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ChainId {
    pub fn default() -> Self {
        Self(1000)
    }
}

#[derive(Debug, PartialEq)]
pub struct ChainSpec {
    pub id: ChainId,
    pub name: &'static str,
    pub hrp: Hrp,
    pub ctr_hrp: Hrp,
}

impl ChainSpec {
    pub fn create(id: ChainId, name: &'static str) -> Result<Self> {
        let hrp = Hrp::parse(&name)?;
        let ctr_hrp = Hrp::parse(&(name.to_owned() + CTR_PREFIX))?;
        Ok(Self { id, name, hrp, ctr_hrp })
    }
}

