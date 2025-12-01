use std::fmt::{Debug, Formatter};
use risc0_zkvm::{Digest, Receipt};
use serde::{Deserialize, Serialize};
use network::handle::{P2pCmdHandle};
use account::keypair::{AccountSigningKey};
use db::handle::DBHandle;
use primitives::hash::Hash32;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db_handle: DBHandle,
    pub cmd_handle: P2pCmdHandle,
    pub sk: AccountSigningKey
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SubmitTxResponse {
    Deploy {
        ctr_addr_str: String,
        image_id: Digest,
        elf_hash: Hash32
    },
    Exec {
        ctr_addr_str: String,
        image_id: Digest,
        elf_hash: Hash32,
        input: Vec<u8>,
        receipt: Receipt,
        answer: Vec<u8>
    }
}

impl Debug for SubmitTxResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "")?;
        writeln!(f, "SubmitTxResponse {{ ")?;
        match self {
            SubmitTxResponse::Deploy { ctr_addr_str, image_id, elf_hash } => {
                writeln!(f, "  ctr_addr_str: {}, ", ctr_addr_str)?;
                writeln!(f, "  image_id: {}, ", image_id.to_string())?;
                writeln!(f, "  elf_hash: {:?}, ", hex::encode(elf_hash))?;
            },
            SubmitTxResponse::Exec { ctr_addr_str, image_id, elf_hash,
                input, receipt, answer  } => {
                writeln!(f, "  ctr_addr_str: {}", ctr_addr_str)?;
                writeln!(f, "  image_id: {}, ", image_id.to_string())?;
                writeln!(f, "  elf_hash: {:?}, ", hex::encode(elf_hash))?;
                writeln!(f, "  input: {:?}", input)?;
                writeln!(f, "  receipt: {:?}, ", receipt)?;
                writeln!(f, "  answer: {:?}, ", answer)?;
            }
        }
        writeln!(f, " }}")    }
}