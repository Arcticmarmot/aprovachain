use std::collections::BTreeMap;
use std::str::FromStr;
use account::executor::ExecutorId;
use platform::config::WorkloadConfig;
use crate::metrics::{Integrity, Metrics, Timeliness};

pub type Weight = u128;
pub type Weights = BTreeMap<ExecutorId, Weight>;


/// 根据 integrity 和 timeliness 计算出权重
pub fn weight_from_metrics(integrity: &Integrity, timeliness: &Timeliness) -> Weight {
    let igt_weight = integrity.score * 0 / 100;
    let tln_weight = timeliness.score * 100 / 100;
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


// - "63d8d75d73303e66217ba9c48d654d6bb7e31ea5f8890459e0a51193dd5e8ab5" # smolensk
// - "40c165755edcc714e564e43e692576951d600a4f416b471a436af5e786d944de" # mecca
// - "88d6d8e53dfae02fac118064124467ec5833b9b0a115d428f7782ccac5862a6f" # aprova-1
// - "c1b1d1aaba1e85d92bc3f9435b110909d9a0e1f73c2c242871a4edefdcd293ea" # aprova-2
// - "f09721496653b0f86a7ef85dbfd8585cb6e560006007389b2283b1a2fc14af2e" # aprova-3
// - "46bfd3164494fea65f58b848b0a702ad931e335354612f1b05259606b930d36d" # aprova-4
// - "d56e1c3294b2d1f9be7d112d9350294448a4c50a67bbc094b781b4bb6a5e750a" # jhc-1
// - "fdeeca3bab2eb1b8edbd60fa4577ef8619ae032f6104a3eb3fd439e4fe4053d2" # jhc-2
// - "1b925e4b25591ff32c845eafd836f47f70f576cf0f268e235cadb353461f9726" # jhc-3
pub fn metrics_to_static_weights(_metrics: &Metrics, workload_config: &WorkloadConfig) -> Weights {
    let mut weights = Weights::new();
    let hetero = &workload_config.heterogeneous;
    match hetero.as_str() {
        "G2C6" => {
            weights.insert(ExecutorId::from_str("d56e1c3294b2d1f9be7d112d9350294448a4c50a67bbc094b781b4bb6a5e750a").unwrap(), 4564);
            weights.insert(ExecutorId::from_str("fdeeca3bab2eb1b8edbd60fa4577ef8619ae032f6104a3eb3fd439e4fe4053d2").unwrap(), 4564);
            weights.insert(ExecutorId::from_str("63d8d75d73303e66217ba9c48d654d6bb7e31ea5f8890459e0a51193dd5e8ab5").unwrap(), 145);
            weights.insert(ExecutorId::from_str("40c165755edcc714e564e43e692576951d600a4f416b471a436af5e786d944de").unwrap(), 145);
            weights.insert(ExecutorId::from_str("88d6d8e53dfae02fac118064124467ec5833b9b0a115d428f7782ccac5862a6f").unwrap(), 145);
            weights.insert(ExecutorId::from_str("c1b1d1aaba1e85d92bc3f9435b110909d9a0e1f73c2c242871a4edefdcd293ea").unwrap(), 145);
            weights.insert(ExecutorId::from_str("f09721496653b0f86a7ef85dbfd8585cb6e560006007389b2283b1a2fc14af2e").unwrap(), 145);
            weights.insert(ExecutorId::from_str("46bfd3164494fea65f58b848b0a702ad931e335354612f1b05259606b930d36d").unwrap(), 145);
        }
        "H2M4" => {
            weights.insert(ExecutorId::from_str("63d8d75d73303e66217ba9c48d654d6bb7e31ea5f8890459e0a51193dd5e8ab5").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("40c165755edcc714e564e43e692576951d600a4f416b471a436af5e786d944de").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("88d6d8e53dfae02fac118064124467ec5833b9b0a115d428f7782ccac5862a6f").unwrap(), 2000);
            weights.insert(ExecutorId::from_str("c1b1d1aaba1e85d92bc3f9435b110909d9a0e1f73c2c242871a4edefdcd293ea").unwrap(), 2000);
            weights.insert(ExecutorId::from_str("f09721496653b0f86a7ef85dbfd8585cb6e560006007389b2283b1a2fc14af2e").unwrap(), 2000);
            weights.insert(ExecutorId::from_str("46bfd3164494fea65f58b848b0a702ad931e335354612f1b05259606b930d36d").unwrap(), 2000);
        }
        "H2M2L2" => {
            weights.insert(ExecutorId::from_str("63d8d75d73303e66217ba9c48d654d6bb7e31ea5f8890459e0a51193dd5e8ab5").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("40c165755edcc714e564e43e692576951d600a4f416b471a436af5e786d944de").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("88d6d8e53dfae02fac118064124467ec5833b9b0a115d428f7782ccac5862a6f").unwrap(), 100);
            weights.insert(ExecutorId::from_str("c1b1d1aaba1e85d92bc3f9435b110909d9a0e1f73c2c242871a4edefdcd293ea").unwrap(), 100);
            weights.insert(ExecutorId::from_str("f09721496653b0f86a7ef85dbfd8585cb6e560006007389b2283b1a2fc14af2e").unwrap(), 2000);
            weights.insert(ExecutorId::from_str("46bfd3164494fea65f58b848b0a702ad931e335354612f1b05259606b930d36d").unwrap(), 2000);
        }
        _ => { panic!("bad heterogeneous") }
    };
    weights
}

pub fn metrics_to_even_weights(_metrics: &Metrics, workload_config: &WorkloadConfig) -> Weights {
    let mut weights = Weights::new();
    let hetero = &workload_config.heterogeneous;
    match hetero.as_str() {
        "G2C6" => {
            weights.insert(ExecutorId::from_str("d56e1c3294b2d1f9be7d112d9350294448a4c50a67bbc094b781b4bb6a5e750a").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("fdeeca3bab2eb1b8edbd60fa4577ef8619ae032f6104a3eb3fd439e4fe4053d2").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("63d8d75d73303e66217ba9c48d654d6bb7e31ea5f8890459e0a51193dd5e8ab5").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("40c165755edcc714e564e43e692576951d600a4f416b471a436af5e786d944de").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("88d6d8e53dfae02fac118064124467ec5833b9b0a115d428f7782ccac5862a6f").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("c1b1d1aaba1e85d92bc3f9435b110909d9a0e1f73c2c242871a4edefdcd293ea").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("f09721496653b0f86a7ef85dbfd8585cb6e560006007389b2283b1a2fc14af2e").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("46bfd3164494fea65f58b848b0a702ad931e335354612f1b05259606b930d36d").unwrap(), 10000);
        }
        "H2M4" => {
            weights.insert(ExecutorId::from_str("d56e1c3294b2d1f9be7d112d9350294448a4c50a67bbc094b781b4bb6a5e750a").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("fdeeca3bab2eb1b8edbd60fa4577ef8619ae032f6104a3eb3fd439e4fe4053d2").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("63d8d75d73303e66217ba9c48d654d6bb7e31ea5f8890459e0a51193dd5e8ab5").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("40c165755edcc714e564e43e692576951d600a4f416b471a436af5e786d944de").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("88d6d8e53dfae02fac118064124467ec5833b9b0a115d428f7782ccac5862a6f").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("c1b1d1aaba1e85d92bc3f9435b110909d9a0e1f73c2c242871a4edefdcd293ea").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("f09721496653b0f86a7ef85dbfd8585cb6e560006007389b2283b1a2fc14af2e").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("46bfd3164494fea65f58b848b0a702ad931e335354612f1b05259606b930d36d").unwrap(), 10000);
        }
        "H2M2L2" => {
            weights.insert(ExecutorId::from_str("d56e1c3294b2d1f9be7d112d9350294448a4c50a67bbc094b781b4bb6a5e750a").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("fdeeca3bab2eb1b8edbd60fa4577ef8619ae032f6104a3eb3fd439e4fe4053d2").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("63d8d75d73303e66217ba9c48d654d6bb7e31ea5f8890459e0a51193dd5e8ab5").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("40c165755edcc714e564e43e692576951d600a4f416b471a436af5e786d944de").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("88d6d8e53dfae02fac118064124467ec5833b9b0a115d428f7782ccac5862a6f").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("c1b1d1aaba1e85d92bc3f9435b110909d9a0e1f73c2c242871a4edefdcd293ea").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("f09721496653b0f86a7ef85dbfd8585cb6e560006007389b2283b1a2fc14af2e").unwrap(), 10000);
            weights.insert(ExecutorId::from_str("46bfd3164494fea65f58b848b0a702ad931e335354612f1b05259606b930d36d").unwrap(), 10000);
        }
        _ => { panic!("bad heterogeneous") }
    }
    weights
}

pub fn total_weights(weights: &Weights) -> u128 {
    weights.values().sum()
}
