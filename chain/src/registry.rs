use crate::spec::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
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
static CUSTOM_MAP: Lazy<RwLock<HashMap<ChainId, Arc<ChainSpec>>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub fn get_or_insert_custom(id: ChainId, name: &'static str) -> Arc<ChainSpec> {
    // 读路径
    if let Some(hit) = CUSTOM_MAP.read().expect("access read lock").get(&id).cloned() {
        return hit;
    }
    let mut w = CUSTOM_MAP.write().expect("access write lock");
    // 双重检查
    w.entry(id).or_insert_with(|| Arc::new(ChainSpec::create(id, name)
        .expect("invalid ChainSpec"))).clone()
}

pub fn get_custom(id: ChainId) -> Option<Arc<ChainSpec>> {
    CUSTOM_MAP.read().expect("access read lock").get(&id).cloned()
}


pub fn by_id(id: ChainId) -> Option<Arc<ChainSpec>> {
    REGISTRY_MAP.get(&id).cloned().or_else(|| get_custom(id))
}

pub fn hrp_by_id(id: ChainId) -> Option<Hrp> {
    by_id(id).map(|spec| spec.hrp)
}
