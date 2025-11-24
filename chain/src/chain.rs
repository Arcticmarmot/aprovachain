use spec::chain::ChainId;
use crate::block::BlockHeader;
use crate::error::{ChainError, Result};

#[derive(Debug)]
pub struct ChainState {
    pub chain_id: ChainId,
    pub tip_header_opt: Option<BlockHeader>,
}

impl ChainState {
    pub fn default(header_opt: Option<BlockHeader>) -> Self {
        Self {
            chain_id: ChainId::default(),
            tip_header_opt: header_opt
        }
    }

    pub fn update(&mut self, next_header: BlockHeader) -> Result<()> {
        match self.tip_header_opt {
            Some(tip_header) => {
                if next_header.parent_hash != tip_header.hash() {
                    return Err(ChainError::InvalidBlock)
                }
                if next_header.height != tip_header.height + 1 {
                    return Err(ChainError::InvalidBlock)
                }
                self.tip_header_opt = Some(next_header);
                Ok(())
            }
            None => {
                if next_header.height != 0 {
                    return Err(ChainError::InvalidBlock)
                }
                self.tip_header_opt = Some(next_header);
                Ok(())
            }
        }
    }
}