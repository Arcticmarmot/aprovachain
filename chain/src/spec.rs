use bech32::Hrp;
use crate::error::ChainError;

type Result<T> = std::result::Result<T, ChainError>;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct ChainId(pub u64);

#[derive(Debug, PartialEq)]
pub struct ChainSpec {
    pub id: ChainId,
    pub name: &'static str,
    pub hrp: Hrp,
}

impl ChainSpec {
    pub fn create(id: ChainId, name: &'static str) -> Result<Self> {
        let hrp = Hrp::parse(&name)?;
        Ok(Self { id, name, hrp })
    }
}

