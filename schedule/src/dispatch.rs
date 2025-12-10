use std::collections::{BTreeMap};
use account::executor::ExecutorId;
use db::handle::DBHandle;
use tx::code::{TxServiceCode, TxServiceCodeMap};
use tx::id::{TxEnvelopeId};
use crate::error::{Result};
use crate::event::EventRecord;
use crate::metrics::*;
use crate::weight::{metrics_to_weights, total_weights};

/// 定义窗口大小
pub const WINDOW_SIZE: usize = 32;
pub const WARMUP_SIZE: usize = 4;

/// 根据一定大小窗口的区块数据计算指标
pub fn compute_metrics_by_window(stats_window: &[TxServiceCodeMap]) -> Metrics {
    let mut metrics = Metrics::new();
    for block_stats in stats_window {
        // 记录每个事件的数量
        let mut event_records: BTreeMap<ExecutorId, EventRecord> = BTreeMap::new();
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
            let (integrity, timeliness): &mut (Integrity, Timeliness) = metrics.entry(exec_id).or_default();
            if let Some(itg_event) = record.compute_integrity_event() {
                integrity.apply_event(itg_event);
            }
            if let Some(tln_event) = record.compute_timeliness_event() {
                timeliness.apply_event(tln_event);
            }
        }
    }
    tracing::info!(target: "schedule::metrics", ?metrics, "metrics");
    metrics
}

pub fn select_executor_for_tx(tx_id: &TxEnvelopeId, metrics: &Metrics) -> Option<ExecutorId> {
    if metrics.is_empty() { return None }
    let weights = metrics_to_weights(&metrics);
    tracing::info!(target: "schedule::weight", ?weights, "weights");
    let rand = pseudo_random_u128(tx_id);
    let total_weight = total_weights(&weights);
    let ticket = rand % total_weight;
    let mut acc = 0;
    for (exec_id, weight) in weights {
        acc += weight;
        if acc >= ticket { return Some(exec_id) }
    }
    None
}

pub fn assign_executor_for_tx(db_handle: &DBHandle, tx_id: &TxEnvelopeId) -> Result<Option<ExecutorId>> {
    let stats_window = db_handle.load_stats_window(WINDOW_SIZE)?;
    // 启动期 slot 长度
    if stats_window.len() >= WARMUP_SIZE {
        let scores = compute_metrics_by_window(&stats_window);
        Ok(select_executor_for_tx(tx_id, &scores))
    } else {
        Ok(None)
    }
}

/// 根据 tx_id 的哈希生成随机数
fn pseudo_random_u128(tx_id: &TxEnvelopeId) -> u128 {
    let mut out = [0u8; 16];
    out.copy_from_slice(&tx_id.hash()[0..16]);
    u128::from_be_bytes(out)
}

