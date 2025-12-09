use std::collections::{HashMap};
use account::executor::ExecutorId;
use db::handle::DBHandle;
use primitives::sha256_join;
use tx::code::{TxServiceCode, TxServiceCodeMap};
use tx::id::{TxEnvelopeId, TxId};
use crate::integrity::Integrity;
use crate::timeliness::Timeliness;
use crate::error::{Result, ScheduleError};
use sha2::{Digest, Sha256};
use primitives::hash::Hash32;

pub const WINDOW_SIZE: usize = 16;

pub fn compute_scores_by_window(stats_window: &[TxServiceCodeMap]) -> HashMap<ExecutorId, (Integrity, Timeliness)> {
    let mut scores = HashMap::new();
    for (_index, block_stats) in stats_window.iter().enumerate() {
        for (_tx_id, (exec_id, code)) in block_stats {
            let (integrity, timeliness): &mut (Integrity, Timeliness) = scores.entry(exec_id.clone()).or_default();
            match code {
                TxServiceCode::Timeout => {
                    timeliness.score -= 10;
                },
                TxServiceCode::FakeReceipt => {
                    integrity.score -= 20;
                },
                TxServiceCode::Success => {
                    timeliness.score += 1;
                    integrity.score += 1;
                },
                TxServiceCode::Conflict => {
                    timeliness.score += 1;
                    integrity.score += 1;
                },
                _ => { }
            }
        }
    }
    scores
}

pub fn weight_from_score(integrity: &Integrity, timeliness: &Timeliness) -> u128 {
    let integrity_weight = (integrity.score) * 3;
    let timeliness_weight = (timeliness.score) * 2;
    let weight = integrity_weight + timeliness_weight;
    weight
}

pub fn select_executor_for_tx(tx_id: &TxEnvelopeId, scores: &HashMap<ExecutorId, (Integrity, Timeliness)>) -> Option<ExecutorId> {
    if scores.is_empty() { return None }
    let mut candidates: Vec<&ExecutorId> = scores.keys().collect();
    candidates.sort();
    let weighted: Vec<(ExecutorId, u128)> = scores
        .iter()
        .map(|(exec_id, (integ, timeli))| {
            let w = weight_from_score(integ, timeli);
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
    let scores = compute_scores_by_window(&stats_window);
    match select_executor_for_tx(tx_id, &scores) {
        Some(exec_id) => Ok(exec_id),
        None => {
            Err(ScheduleError::EmptyStats)
        }
    }
}

fn pseudo_random_u128(tx_id: &TxEnvelopeId) -> u128 {
    let hash: Hash32 = sha256_join!(tx_id.0);
    let mut out = [0u8; 16];
    out.copy_from_slice(&hash[0..16]);
    u128::from_be_bytes(out)
}

#[test]
fn test_assign() {
    let db_handle = DBHandle::new().unwrap();
    let tx_id = TxEnvelopeId::new([1u8; 32]);
    let exec_id = assign_executor_for_tx(&db_handle, &tx_id).unwrap();
    println!("{:?}", exec_id);
}