use std::collections::{BTreeMap};
use account::executor::ExecutorId;
use db::handle::DBHandle;
use tx::code::{TxServiceCode, TxServiceCodeMap};
use tx::id::{TxEnvelopeId};
use crate::error::{Result, ScheduleError};
use crate::metrics::*;

pub const WINDOW_SIZE: usize = 16;
/// 根据一定大小窗口的区块数据计算指标
pub fn compute_metrics_by_window(stats_window: &[TxServiceCodeMap]) -> Metrics {
    let mut metrics = Metrics::new();

    for block_stats in stats_window {
        // 记录每个事件的数量
        let mut event_records: BTreeMap<ExecutorId, BlockEventRecord> = BTreeMap::new();
        for (_, (exec_id, code)) in block_stats {
            match code {
                TxServiceCode::Success | TxServiceCode::Conflict => {
                    event_records.entry(exec_id.clone()).or_default().valid += 1;
                },
                TxServiceCode::Timeout => {
                    event_records.entry(exec_id.clone()).or_default().timeout += 1;
                },
                TxServiceCode::FakeReceipt => {
                    event_records.entry(exec_id.clone()).or_default().fake_receipt += 1;
                },
                _ => { }
            }
        }
        for (exec_id, record) in event_records {
            // TODO: total_event_num() 为 0 的情况
            if record.total_event_num() == 0 { break; }
            let (integrity, timeliness): &mut (Integrity, Timeliness) = metrics.entry(exec_id.clone()).or_default();
            let igt_score = integrity.score;
            let tln_score = timeliness.score;
            integrity.score = ema_new_val(igt_score, record.compute_integrity_event());
            timeliness.score = ema_new_val(tln_score, record.compute_timeliness_event());
        }
    }
    metrics
}

pub fn select_executor_for_tx(tx_id: &TxEnvelopeId, metrics: &Metrics) -> Option<ExecutorId> {
    if metrics.is_empty() { return None }
    let weighted: Vec<(ExecutorId, u128)> = metrics
        .iter()
        .map(|(exec_id, (integ, timeli))| {
            let w = weight_from_metrics(integ, timeli);
            (exec_id.clone(), w)
        })
        .collect();
    if weighted.is_empty() { return None; }
    let rand = pseudo_random_u128(tx_id);
    let mut total_weight = 0;
    for (_, w) in &weighted {
        total_weight += *w;
    }
    let ticket = rand % total_weight;
    let mut acc = 0;
    for (exec_id, w) in weighted {
        acc += w;
        if acc > ticket {
            return Some(exec_id)
        }
    }
    None
}

pub fn assign_executor_for_tx(db_handle: &DBHandle, tx_id: &TxEnvelopeId) -> Result<ExecutorId> {
    let stats_window = db_handle.load_stats_window(WINDOW_SIZE)?;
    let scores = compute_metrics_by_window(&stats_window);
    match select_executor_for_tx(tx_id, &scores) {
        Some(exec_id) => Ok(exec_id),
        None => {
            Err(ScheduleError::EmptyStats)
        }
    }
}


