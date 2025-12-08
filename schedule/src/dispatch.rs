use std::collections::{HashMap};
use account::executor::ExecutorId;
use db::handle::DBHandle;
use tx::code::{TxServiceCode, TxServiceCodeMap};
use tx::id::TxId;
use crate::integrity::Integrity;
use crate::timeliness::Timeliness;
use crate::error::Result;
pub const WINDOW_SIZE: usize = 16;

pub fn compute_scores_by_window(stats_window: &Vec<TxServiceCodeMap>) -> HashMap<ExecutorId, (Integrity, Timeliness)> {
    let mut scores = HashMap::new();
    for (_index, block_log) in stats_window.iter().enumerate() {
        for (_tx_id, (exec_id, code)) in block_log {
            let (integ, timeli): &mut (Integrity, Timeliness) = scores.entry(exec_id.clone()).or_default();
            match code {
                TxServiceCode::Timeout => {
                    timeli.score -= 10;
                },
                TxServiceCode::FakeReceipt => {
                    integ.score -= 20;
                },
                TxServiceCode::Success => {
                    timeli.score += 1;
                    integ.score += 1;
                },
                TxServiceCode::Conflict => {
                    timeli.score += 1;
                    integ.score += 1;
                },
                _ => { }
            }
        }
    }
    scores
}

pub fn select_executor_for_tx(tx_id: TxId, scores: HashMap<ExecutorId, (Integrity, Timeliness)>) -> ExecutorId {
    todo!()
}

pub fn assign_executor_for_tx(db_handle: &DBHandle, tx_id: TxId) -> Result<ExecutorId> {
    let stats_window = db_handle.load_stats_window(WINDOW_SIZE)?;
    let scores = compute_scores_by_window(&stats_window);
    Ok(select_executor_for_tx(tx_id, scores))
}