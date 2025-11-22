use spec::chain::ChainId;
use crate::block::BlockHeader;
use crate::error::{ChainError, Result};

#[derive(Debug)]
pub struct ChainState {
    pub chain_id: ChainId,
    pub tip_header: BlockHeader,
}

impl ChainState {
    pub fn default(header: BlockHeader) -> Self {
        Self {
            chain_id: ChainId::default(),
            tip_header: header
        }
    }

    pub fn update(&mut self, next_header: BlockHeader) -> Result<()> {
        if next_header.parent_hash != self.tip_header.hash() {
            return Err(ChainError::InvalidBlock)
        }
        self.tip_header = next_header;
        Ok(())
    }
}