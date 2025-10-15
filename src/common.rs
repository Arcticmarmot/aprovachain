use std::net::{IpAddr, UdpSocket};
use serde_json::json;
use sha2::{Digest, Sha256};
use crate::app::{Chain, DIFFICULTY_PREFIX};

pub const LOCAL_IP_ADDR: &'static str = "0.0.0.0";
pub const BROADCAST_IP_ADDR: &'static str = "192.168.1.255";
pub const UDP_DISC_PORT: u16 = 9000;
pub const UDP_LISTENER_PORT: u16 = 9001;
pub const BROADCAST_INTERVAL_SECS: u64 = 5;

pub fn calc_hash(id: u64, timestamp: i64, pre_hash: &str, data: &str, nonce: u64) -> String {
    let data = json!({
        "id": id,
        "previous_hash": pre_hash,
        "timestamp": timestamp,
        "data": data,
        "nonce": nonce
    });
    let mut hasher = Sha256::new();
    hasher.update(data.to_string().as_bytes());
    let hash_bytes = hasher.finalize().as_slice().to_vec();
    hex::encode(hash_bytes)
}

pub fn calc_peer_hash(ip_addr: IpAddr, port: u16) -> String {
    let data = json!({
        "ip_addr": ip_addr,
        "port": port,
    });
    let mut hasher = Sha256::new();
    hasher.update(data.to_string().as_bytes());
    let hash_bytes = hasher.finalize().as_slice().to_vec();
    hex::encode(hash_bytes)
}

pub fn mine(id: u64, timestamp: i64, pre_hash: &str, data: &str) -> (u64, String) {
    println!("start mining...");
    let mut nonce = 0;
    loop {
        let hash = calc_hash(id, timestamp, pre_hash, data, nonce);
        println!("nonce: {}, hash: {}", nonce, hash);
        if hash.starts_with(DIFFICULTY_PREFIX) {
            return (nonce, hash)
        }
        nonce += 1;
    }
}

pub fn chose_chain<'a>(left: &'a Chain, right: &'a Chain) -> &'a Chain {
    let is_left_chain_valid = left.is_chain_valid();
    let is_right_chain_valid = right.is_chain_valid();
    if is_left_chain_valid && is_right_chain_valid {
        return if left.blocks.len() > right.blocks.len() {
            left
        } else {
            right
        }
    }
    if is_left_chain_valid { left } else { right }
}

pub fn get_default_outbound_ip() -> Option<IpAddr> {
    // 这个地址不会被实际连接/发送数据，只用于内核选择出口接口，从而能读取 local_addr。
    // 8.8.8.8:80 是常见选择（任何可达的公网 IP 都可）
    match UdpSocket::bind("0.0.0.0:0") {
        Ok(sock) => {
            if sock.connect("8.8.8.8:80").is_ok() {
                if let Ok(local) = sock.local_addr() {
                    return Some(local.ip());
                }
            }
            None
        }
        Err(_) => None,
    }
}