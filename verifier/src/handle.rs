use anyhow::Result;
use db::handle::DBHandle;
use engine::verify::verify_and_apply_block;

pub fn handle_block_received(db_handle: &DBHandle, block_bytes: Vec<u8>) -> Result<()> {
    verify_and_apply_block(db_handle, block_bytes)
}