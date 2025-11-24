use anyhow::Result;
use chain::block::Block;

pub fn handle_tx_received(tx_bytes: Vec<u8>) -> Result<()> {
    Ok(())
}

pub fn handle_block_received(block_bytes: Vec<u8>) -> Result<()> {
    let block = Block::try_decode_bcs(&block_bytes)?;
    Ok(())
}