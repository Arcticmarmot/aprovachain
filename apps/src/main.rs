use clap::Parser;
use anyhow::{Result};
use apps::bootstrap::{init_env, init_logging};
use apps::handler::{parse_tx_args, send_envelope, TxArgs};

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