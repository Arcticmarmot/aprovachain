use bech32::Hrp;
use crate::error::ChainError;

type Result<T> = std::result::Result<T, ChainError>;
pub static CTR_PREFIX: &'static str = "ctr";

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ChainId(pub u64);

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

