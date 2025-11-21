use risc0_zkvm::{Digest, Receipt};
use serde::{Deserialize, Serialize};
use account::address::ChainAddrBytes;
use network::handle::{P2pCmdHandle};
use account::keypair::{AccountSigningKey};
use primitives::hash::Hash32;

#[derive(Debug, Clone)]
pub struct AppState {
    pub cmd_handle: P2pCmdHandle,
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