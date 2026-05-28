use std::collections::BTreeMap;
use anyhow::{anyhow, Context, Result};
use account::executor::ExecutorId;
use chain::block::OrderedBlock;
use db::handle::DBHandle;
use engine::execute::{pre_exec_tx, verify_and_build_envelope};
use engine::validate::verify_and_apply_block;
use platform::config::{DispatchConfig, ProveMode, ValidateMode, WorkloadConfig};
use schedule::dispatch::assign_executor_for_tx;
use task::schedule::TaskSchedule;
use tx::intent::TxPayload;

pub async fn on_envelope_received(schedule: TaskSchedule, self_exec_id: ExecutorId,
                                  envelope_bytes: Vec<u8>, dispatch_config: DispatchConfig, workload_config: WorkloadConfig) -> Result<()> {
    // 加载状态信息
    let db_handle = DBHandle::new()?;

    // 从字节数组构造 TxEnvelope
    let envelope = verify_and_build_envelope(&envelope_bytes)?;
    let send_ts = envelope.intent.timestamp;
    let envelope_id = envelope.tx_id();

    let exec_id = match assign_executor_for_tx(&db_handle, &envelope_id, send_ts, dispatch_config, workload_config)? {
        Some(exec_id) => { exec_id },
        None => {
            tracing::info!(target: "executor::event", %envelope_id, "no metrics yet, fall back to self as executor");
            self_exec_id
        }
    };
    tracing::info!(target:"executor::event", %exec_id, "executor id");

    if exec_id != self_exec_id {
        tracing::info!(target:"executor::event", "none of my business");
        return Ok(());
    }

    let TxPayload::Exec { ctr_addr_str, input, access_set } = &envelope.intent.payload else {
        tracing::debug!(target: "executor::event", %envelope_id, "payload is not Exec");
        return Ok(());
    };
    tracing::info!(target:"executor::event", %envelope_id, "handle envelope myself");
    let send_height = db_handle
        .load_ts_height(send_ts)?
        .ok_or_else(|| anyhow!("genesis ts not found for send_ts={send_ts}"))?;

    let scale = pre_exec_tx(&db_handle, &envelope, ctr_addr_str, input, access_set)
        .context("pre_exec_tx failed")?;
    schedule.push(envelope, scale, send_height).await;
    Ok(())
}

pub async fn on_block_received(db_handle: &DBHandle, block_bytes: Vec<u8>,
                               pending_blocks: &mut BTreeMap<u128, Vec<u8>>,
                               prove_mode: ProveMode,
                               validate_mode: ValidateMode,
                               dispatch_config: DispatchConfig,
                               workload_config: WorkloadConfig) -> Result<()> {
    let block = OrderedBlock::try_decode_bcs(&block_bytes)?;
    let height = block.header.height;

    let expected = expected_height(db_handle)?;

    // 旧块 / 重复块，直接忽略
    if height < expected {
        return Ok(());
    }

    // 当前块或未来块，先缓存
    pending_blocks.entry(height).or_insert(block_bytes);

    // 只要缓存里有 expected height，就连续处理
    loop {
        let expected = expected_height(db_handle)?;

        let Some(next_block_bytes) = pending_blocks.remove(&expected) else {
            break;
        };

        verify_and_apply_block(
            db_handle,
            next_block_bytes,
            prove_mode.clone(),
            validate_mode.clone(),
            dispatch_config.clone(),
            workload_config.clone(),
        ).await?;
    }

    Ok(())
}

fn expected_height(db_handle: &DBHandle) -> Result<u128> {
    Ok(match db_handle.load_chain_state()? {
        Some(header) => header.height + 1,
        None => 0,
    })
}