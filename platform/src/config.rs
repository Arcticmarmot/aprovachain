use std::fs;
use std::path::{Path, PathBuf};
use serde::Deserialize;

fn default_chain_id() -> u64 { 1000 }
fn default_slot_secs() -> u64 { 12 }

#[derive(Debug, Clone, Deserialize)]
pub struct ProveConfig {
    pub mode: String,   // "real" | "fake"
    pub scheme: String,
    pub latency: u64,
    pub offset: u64
}

#[derive(Debug, Clone)]
pub enum ProveMode {
    Native { scheme: String },
    Simulate { scheme: String, latency: u64, offset: u64 }
}


#[derive(Debug, Clone, Deserialize)]
pub struct DispatchConfig {
    pub window_size: u32,
    pub ema_k: u32,
    pub score: DispatchScoreConfig,
}



#[derive(Debug, Clone, Deserialize)]
pub struct DispatchScoreConfig {
    pub timeliness: TimelinessScoreConfig,
    pub integrity: IntegrityScoreConfig,
}


#[derive(Debug, Clone, Deserialize)]
pub struct TimelinessScoreConfig {
    pub timeout: u128,
}


#[derive(Debug, Clone, Deserialize)]
pub struct IntegrityScoreConfig {
    pub fake: u128,
}


#[derive(Debug, Clone, Deserialize)]
pub struct QueueConfig {
    pub discipline: String, // fcfs | spt | edf | spt_edf | edf_spt
}


#[derive(Debug, Clone, Deserialize)]
pub struct BaseConfig {
    #[serde(default="default_chain_id")]
    pub chain_id: u64,

    #[serde(default="default_slot_secs")]
    pub slot_secs: u64,
    
    pub consensus: ConsensusConfig,

    pub prove: ProveConfig,

    pub dispatch: DispatchConfig,

    pub queue: QueueConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConsensusConfig {
    pub protocol: String,
    pub tx_capacity: usize,
    pub leader_id: String,
    pub members: Vec<String>,
}

pub fn load_base_config() -> BaseConfig {
    let path = default_base_yaml_path();
    load_config_from(&path)
}

fn load_config_from(path: impl AsRef<Path>) -> BaseConfig {
    let s = fs::read_to_string(path.as_ref()).expect("read config file failed");
    let cfg: BaseConfig = serde_yaml::from_str(&s).expect("load base config failed");
    cfg
}

fn default_base_yaml_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.join("..").join("config").join("base.yaml")
}