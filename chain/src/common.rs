use log::info;
use serde_json::json;
use sha2::{Digest, Sha256};
use crate::app::{Block, DIFFICULTY_PREFIX};

pub fn calc_hash(id: u64, timestamp: i64, previous_hash: &str, data: &str, nonce: u64) -> String {
    let data = json!({
        "id": id,
        "previous_hash": previous_hash,
        "timestamp": timestamp,
        "data": data,
        "nonce": nonce
    });
    let mut hasher = Sha256::new();
    hasher.update(data.to_string().as_bytes());
    let hash_bytes = hasher.finalize().as_slice().to_vec();
    hex::encode(hash_bytes)
}

pub fn mine(id: u64, timestamp: i64, previous_hash: &str, data: &str) -> (u64, String) {
    println!("start mining...");
    let mut nonce = 0;

    loop {
        let hash = calc_hash(id, timestamp, previous_hash, data, nonce);
        println!("{}", nonce);
        println!("{}", hash);
        println!("====================================");
        if hash.starts_with(DIFFICULTY_PREFIX) {
            return (nonce, hash)
        }
        nonce += 1;
    }
}