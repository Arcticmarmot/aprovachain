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

    let tx_envelope = parse_tx_args(args)?;

    send_envelope(tx_envelope).await
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn deploy_contract() {
        unsafe {
            std::env::set_var("APROVA_RPC_URL", "http://127.0.0.1:9999");
        }

    }
}
