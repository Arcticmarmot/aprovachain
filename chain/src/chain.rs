use spec::chain::ChainId;
use crate::block::Block;

pub struct Chain {
    chain_id: ChainId,
    current: Block
}