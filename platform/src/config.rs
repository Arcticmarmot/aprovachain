use std::fs;
use std::path::{Path, PathBuf};
use serde::Deserialize;

fn default_chain_id() -> u64 { 1000 }
fn default_slot_secs() -> u32 {
    12
}

fn default_tx_capacity() -> u32 {
    100
}

fn default_consensus_config() -> ConsensusConfig {
    ConsensusConfig {
        protocol: "SOLO".to_string(),
        leader_id: "".to_string(),
        members: Vec::new()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BaseConfig {
    #[serde(default="default_chain_id")]
    pub chain_id: u64,
    #[serde(default="default_slot_secs")]
    pub slot_secs: u32,
    #[serde(default="default_tx_capacity")]
    pub tx_capacity: u32,
    #[serde(default="default_consensus_config")]
    pub consensus: ConsensusConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConsensusConfig {
    pub protocol: String,
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