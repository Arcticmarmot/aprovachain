mod bootstrap;
mod handler;

use clap::Parser;
use anyhow::{Result};
use crate::bootstrap::{init_env, init_logging};
use crate::handler::{parse_tx_args, send_envelope, TxArgs};

#[tokio::main]
async fn main() -> Result<()> {
    init_logging()?;

    init_env()?;
    tracing::info!("Aprova app init succeeded ...");

    let args = TxArgs::parse();
    tracing::info!("TxArgs:{:?}", args);

    let tx_envelope = parse_tx_args(&args)?;

    send_envelope(tx_envelope).await
}


#[cfg(test)]
mod tests {
    use std::fs;
    use contracts::{UAV_ELF, UAV_ID};
    use tx::tx_intent::TxPayload;
    use super::*;
    #[test]
    fn deploy_contract_by_json() {
        unsafe {
            std::env::set_var("APROVA_RPC_URL", "http://127.0.0.1:9999");
        }

        // let elf_file = UAV_ELF;
        // let elf_id = UAV_ID;
        // tracing::info!("{:?}", elf_id);
        // tracing::info!("LEN: {}", elf_file.len());
        // let payload = TxPayload::Deploy { source: Vec::from(elf_file) };

        // let bin = read_bin_file("/home/woolf/aprova/aprova/apps/ELF/aprova-guest.bin").unwrap();
        // let payload = TxPayload::Deploy { source: bin };
        // println!("{:?}", payload);
    }
}
