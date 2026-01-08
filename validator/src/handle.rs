use db::handle::DBHandle;
use engine::validate::verify_and_apply_block;
use platform::config::{DispatchConfig, ValidateMode};

pub async fn on_block_received(db_handle: &DBHandle, block_bytes: Vec<u8>, validate_mode: ValidateMode, dispatch_config: DispatchConfig) -> anyhow::Result<()> {
    verify_and_apply_block(db_handle, block_bytes, validate_mode, dispatch_config).await
}