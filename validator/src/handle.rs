use db::handle::DBHandle;
use engine::validate::verify_and_apply_block;
use platform::config::{DispatchConfig, ProveMode, ValidateMode, WorkloadConfig};

pub async fn on_block_received(db_handle: &DBHandle, block_bytes: Vec<u8>, prove_mode: ProveMode,
                               validate_mode: ValidateMode, dispatch_config: DispatchConfig, workload_config: WorkloadConfig) -> anyhow::Result<()> {
    verify_and_apply_block(db_handle, block_bytes, prove_mode, validate_mode, dispatch_config, workload_config).await
}