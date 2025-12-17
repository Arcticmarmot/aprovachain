use anyhow::{anyhow, Result};
use account::executor::ExecutorId;
use db::handle::DBHandle;
use engine::execute::verify_and_build_envelope;
use engine::verify::verify_and_apply_block;
use schedule::dispatch::assign_executor_for_tx;
use task::schedule::TaskSchedule;

pub async fn on_envelope_received(schedule: TaskSchedule, self_exec_id: ExecutorId, envelope_bytes: Vec<u8>) -> Result<()> {
    // 加载状态信息
    let db_handle = DBHandle::new()?;

    // 从字节数组构造 TxEnvelope
    let envelope = verify_and_build_envelope(&envelope_bytes)?;
    let envelope_id = envelope.tx_id();

    let exec_id = match assign_executor_for_tx(&db_handle, &envelope_id, envelope.intent.timestamp)? {
        Some(exec_id) => { exec_id },
        None => {
            tracing::info!(target: "executor::event", %envelope_id, "no metrics yet, fall back to self as executor");
            self_exec_id
        }
    };
    tracing::info!(target:"executor::event", %exec_id, "executor id");

    if exec_id == self_exec_id {
        tracing::info!(target:"executor::event", "I will do it");

        // 交易放入任务队列
        let send_height = db_handle.load_ts_height(envelope.intent.timestamp)?
            .ok_or_else(|| anyhow!("genesis ts not found"))?;
        schedule.push(envelope_bytes.as_ref(), envelope.intent.scale, send_height).await;
    } else {
        tracing::info!(target:"executor::event", "none of my business");
    }
    Ok(())
}

pub fn on_block_received(db_handle: &DBHandle, block_bytes: Vec<u8>) -> Result<()> {
    verify_and_apply_block(db_handle, block_bytes)
}