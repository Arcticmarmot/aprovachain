use crate::spec::*;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};
use once_cell::sync::Lazy;


struct ChainSpecConf {
    pub id: ChainId,
    pub name: &'static str,
}

static KNOWN_CHAIN: &[ChainSpecConf] = &[
    ChainSpecConf { id: ChainId(1000), name: "mainnet" },
    ChainSpecConf { id: ChainId(2000), name: "testnet" }
];

static MAINNET: OnceLock<Arc<ChainSpec>> = OnceLock::new();
static TESTNET: OnceLock<Arc<ChainSpec>> = OnceLock::new();
static CUSTOM_NETS: Lazy<RwLock<HashMap<ChainId, Arc<ChainSpec>>>> = Lazy::new(|| RwLock::new(HashMap::new()));
static REGISTRY_CHAIN: Lazy<HashMap<ChainId, Arc<ChainSpec>>> = Lazy::new(|| {
    let mut m = HashMap::with_capacity(KNOWN_CHAIN.len());
    m
});

/// OnceLock 方案
fn init_net(net: &'static OnceLock<Arc<ChainSpec>>, chain_id: ChainId, name: &'static str) -> Arc<ChainSpec> {
    net.get_or_init(|| {
        Arc::new(ChainSpec::create(chain_id, name).expect(&format!("hardcoded HRP {} should be valid", name)))
    }).clone()
}

fn mainnet() -> Arc<ChainSpec> {
    init_net(&MAINNET, ChainId(1000), "mainnet")
}

fn testnet() -> Arc<ChainSpec> {
    init_net(&TESTNET, ChainId(2000), "testnet")
}

fn get_or_insert_custom(id: ChainId) -> Arc<ChainSpec> {
    // 读路径
    if let Some(hit) = CUSTOM_NETS.read().expect("access read lock").get(&id).cloned() {
        return hit;
    }
    let mut w = CUSTOM_NETS.write().expect("access write lock");
    // 双重检查
    w.entry(id).or_insert_with(|| Arc::new(ChainSpec::create(id, "custom")
        .expect("invalid custom net"))).clone()
}

pub fn by_id(id: ChainId) -> Arc<ChainSpec> {
    if let Some(hit) = REGISTRY_CHAIN.iter().cloned().find(|c| c.id == id ) {
        return hit;
    }
    get_or_insert_custom(id)
}