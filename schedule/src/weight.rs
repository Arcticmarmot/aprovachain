use std::collections::BTreeMap;
use account::executor::ExecutorId;
use crate::metrics::{Integrity, Metrics, Timeliness};

pub type Weight = u128;
pub type Weights = BTreeMap<ExecutorId, Weight>;


/// 根据 integrity 和 timeliness 计算出权重
pub fn weight_from_metrics(integrity: &Integrity, timeliness: &Timeliness) -> Weight {
    let igt_weight = integrity.score * 40 / 100;
    let tln_weight = timeliness.score * 60 / 100;
    let weight = igt_weight + tln_weight;
    weight.max(1)
}

pub fn metrics_to_weights(metrics: &Metrics) -> Weights {
    let mut weights = Weights::new();
    for (exec_id, (itg, tlm)) in metrics {
        weights.insert(exec_id.clone(), weight_from_metrics(itg, tlm));
    }
    weights
}

pub fn total_weights(weights: &Weights) -> u128 {
    weights.values().sum()
}
