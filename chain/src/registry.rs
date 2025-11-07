use crate::spec::*;
use std::collections::HashMap;
use std::sync::{Arc};
use parking_lot::{RwLock, RwLockUpgradableReadGuard};
use bech32::Hrp;
use once_cell::sync::Lazy;

struct ChainSpecConf {
    pub id: ChainId,
    pub name: &'static str,
}

static KNOWN_CHAIN: &[ChainSpecConf] = &[
    ChainSpecConf { id: ChainId(1000), name: "main" },
    ChainSpecConf { id: ChainId(2000), name: "test" },
];

static REGISTRY_MAP: Lazy<HashMap<ChainId, Arc<ChainSpec>>> = Lazy::new(|| {
    let mut map = HashMap::with_capacity(KNOWN_CHAIN.len());
    for &ChainSpecConf { id, name } in KNOWN_CHAIN {
        let spec = ChainSpec::create(id, name)
            .unwrap_or_else(|e| panic!("invalid hardcoded chain {id:?} {name}: {e}"));
        assert!(map.insert(id, Arc::new(spec)).is_none(), "duplicated ChainId: {id:?}");
    }
    map
});

/// parking_lot::RwLock 无毒化 没有PoisonError干扰，需要保证 无半成品更新
/// 先构造后插入模式：所有可能失败/会分配/会解析的步骤在锁外完成；进入临界区只做 insert/swap/replace
static CUSTOM_MAP: Lazy<RwLock<HashMap<ChainId, Arc<ChainSpec>>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub fn get_or_insert_custom(id: ChainId, name: &'static str) -> Arc<ChainSpec> {
    // 获取可升级的读锁
    let upg = CUSTOM_MAP.upgradable_read();
    if let Some(hit) = upg.get(&id).cloned() {
        return hit;
    }
    // 在升级前把可能 panic 的工作做完，避免持锁期间出错
    let spec = Arc::new(ChainSpec::create(id, name).expect("invalid ChainSpec"));
    // 从读锁升级到写锁
    let mut w = RwLockUpgradableReadGuard::upgrade(upg);
    // 双重检查
    w.entry(id).or_insert_with(|| spec).clone()
}

pub fn get_custom(id: ChainId) -> Option<Arc<ChainSpec>> {
    CUSTOM_MAP.read().get(&id).cloned()
}


pub fn by_id(id: ChainId) -> Option<Arc<ChainSpec>> {
    REGISTRY_MAP.get(&id).cloned().or_else(|| get_custom(id))
}
/// TODO: change to bech32m
pub fn hrp_by_id(id: ChainId) -> Option<Hrp> {
    by_id(id).map(|spec| spec.hrp)
}
