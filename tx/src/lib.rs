use std::time::{SystemTime, UNIX_EPOCH};
use ed25519_dalek::Signature;
use rand_core::{OsRng, TryRngCore};
use serde::{Deserialize, Serialize};
use account::address::{AccountAddress, UserAddress};
use account::keypair::{AccountSigningKey, AccountVerifyingKey};

fn now_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).expect("accessing timestamp").as_secs()
}

fn random_u128() -> u128 {
    let mut bytes = [0u8; 16];
    OsRng.try_fill_bytes(&mut bytes).expect("OS RNG unavailable");
    u128::from_le_bytes(bytes)
}

impl TxBody {
    pub fn new(addr: AccountAddress<UserAddress>, vk: AccountVerifyingKey, payload: TxPayload) -> Self {
        let nonce = random_u128();
        let timestamp = now_timestamp();
        Self{
            nonce,
            timestamp,
            address: addr,
            verifying_key: vk,
            payload,
        }
    }
}

impl Tx {
    pub fn new(body: TxBody, sk: AccountSigningKey) -> Self {
        let tx_id = "tx_id".to_string();
        Self {
            tx_id: tx_id.clone(),
            body,
            signature: sk.sign(&tx_id.as_bytes())
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TxPayload {
    Exec { image_id: String, input:Vec<u8> }
}

#[derive(Debug)]
pub struct TxBody {
    pub nonce: u128,
    pub address: AccountAddress<UserAddress>,
    pub verifying_key: AccountVerifyingKey,
    pub timestamp: u64,
    pub payload: TxPayload,
}
#[derive(Debug)]
pub struct Tx {
    pub tx_id: String,
    pub body: TxBody,
    pub signature: Signature,
}

