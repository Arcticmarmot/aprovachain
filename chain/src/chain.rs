use primitives::hash::Hash32;
use spec::chain::ChainId;
use crate::block::BlockHeader;

pub struct ChainState {
    pub chain_id: ChainId,
    pub tip_header: BlockHeader,
}