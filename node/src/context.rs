use risc0_zkvm::{Digest, Receipt};
use serde::{Deserialize, Serialize};
use account::address::ChainAddrBytes;
use network::handle::P2pHandle;
use account::keypair::{AccountSigningKey};
use primitives::hash::Hash32;

#[derive(Debug, Clone)]
pub struct AppState {
    pub p2p_handle: P2pHandle,
    pub sk: AccountSigningKey
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SubmitTxResponse {
    Deploy {
        ctr_addr: ChainAddrBytes,
        image_id: Digest,
        elf_hash: Hash32
    },
    Exec {
        ctr_addr: ChainAddrBytes,
        image_id: Digest,
        elf_hash: Hash32,
        input: Vec<u8>,
        receipt: Receipt
    }
}