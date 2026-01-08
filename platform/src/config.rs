use std::fs;
use std::path::{Path, PathBuf};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct BaseConfig {
    pub chain_id: u64,

    pub slot_secs: u64,

    pub enable_prove_receipt_recording: bool,

    pub prove_receipt_csv: String,

    pub enable_verify_receipt_recording: bool,

    pub verify_receipt_csv: String,

    pub enable_validate_block_recording: bool,

    pub validate_block_csv: String,

    pub workload: WorkloadConfig,

    pub consensus: ConsensusConfig,

    pub prove: ProveConfig,

    pub dispatch: DispatchConfig,

    pub queue: QueueConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkloadConfig {
    pub accounts_num: usize,
    pub tx_num: usize,
    pub init_balance: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProveConfig {
    pub mode: String,
    pub scheme: String,
    pub latency: u64,
    pub offset: u64
}

#[derive(Debug, Clone)]
pub enum ProveMode {
    Native { 
        scheme: String,
        enable_prove_receipt_recording: bool,
        prove_receipt_csv: String,
    },
    Simulate { 
        scheme: String, 
        latency: u64, 
        offset: u64,
        enable_prove_receipt_recording: bool,
        prove_receipt_csv: String
    }
}

#[derive(Debug, Clone)]
pub struct ValidateMode {
    pub prove_scheme: String,
    pub simulate_size: usize,
    pub simulate_size_array: Vec<usize>,
    pub enable_verify_receipt_recording: bool,
    pub verify_receipt_csv: String,
    pub enable_validate_block_recording: bool,
    pub validate_block_csv: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DispatchConfig {
    pub mode: String,
    pub window_size: usize,
    pub warmup_size: usize,
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
pub struct ConsensusConfig {
    pub mode: String,
    pub simulate_size: usize,
    pub simulate_size_array: Vec<usize>,
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