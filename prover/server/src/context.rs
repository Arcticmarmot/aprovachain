use std::fmt::{Debug, Formatter};
use risc0_zkvm::{Digest, Receipt};
use serde::{Deserialize, Serialize};
use account::executor::ExecutorId;
use network::handle::{P2pCmdHandle};
use account::keypair::{AccountSigningKey};
use apps::ctr_io::AccessSet;
use db::handle::DBHandle;
use platform::config::{DispatchConfig, ProveMode};
use primitives::hash::Hash32;
use task::schedule::TaskSchedule;

#[derive(Debug, Clone)]
pub struct AppState {
    pub db_handle: DBHandle,
    pub cmd_handle: P2pCmdHandle,
    pub sk: AccountSigningKey,
    pub schedule: TaskSchedule,
    pub prove_mode: ProveMode,
    pub dispatch_config: DispatchConfig,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SubmitTxResponse {
    Exec {
        ctr_addr_str: String,
        access_set: AccessSet,
        input: Vec<u8>,
        receipt: Receipt,
        answer: Vec<u8>
    },
    Deploy {
        ctr_addr_str: String,
        image_id: Digest,
        elf_hash: Hash32
    },
    Update {
        ctr_addr_str: String,
        image_id: Digest,
        elf_hash: Hash32
    },
    Pending {
        ctr_addr_str: String,
        executor_id: ExecutorId,
    },
    CatalogStore,
    Invalid {
        message: String,
    }
}

impl Debug for SubmitTxResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "")?;
        writeln!(f, "SubmitTxResponse {{ ")?;
        match self {
            SubmitTxResponse::Exec { ctr_addr_str, access_set,
                input, receipt, answer  } => {
                writeln!(f, "  ctr_addr_str: {}", ctr_addr_str)?;
                writeln!(f, "  access_set: {:?}, ", access_set)?;
                writeln!(f, "  input: {:?}", input)?;
                writeln!(f, "  receipt: {:?}, ", receipt)?;
                writeln!(f, "  answer: {:?}, ", answer)?;
            },
            SubmitTxResponse::Deploy { ctr_addr_str, image_id, elf_hash } => {
                writeln!(f, "  ctr_addr_str: {}, ", ctr_addr_str)?;
                writeln!(f, "  image_id: {}, ", image_id.to_string())?;
                writeln!(f, "  elf_hash: {:?}, ", hex::encode(elf_hash))?;
            },
            SubmitTxResponse::Update { ctr_addr_str, image_id, elf_hash } => {
                writeln!(f, "  ctr_addr_str: {}, ", ctr_addr_str)?;
                writeln!(f, "  image_id: {}, ", image_id.to_string())?;
                writeln!(f, "  elf_hash: {:?}, ", hex::encode(elf_hash))?;
            },
            SubmitTxResponse::Pending { ctr_addr_str, executor_id } => {
                writeln!(f, "  ctr_addr_str: {}, ", ctr_addr_str)?;
                writeln!(f, "  executor_id: {}, ", executor_id)?;
            },
            SubmitTxResponse::CatalogStore => {
                writeln!(f, "  catalog store ")?;
            }
            SubmitTxResponse::Invalid { message } => {
                writeln!(f, "  message: {}, ", message)?;
            }
        }
        writeln!(f, " }}")    }
}