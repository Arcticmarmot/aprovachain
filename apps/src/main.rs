use clap::Parser;
use anyhow::{Result};
use apps::bootstrap::{init_env, init_logging};
use apps::handler::{build_envelope_wire, parse_tx_args, send_envelope, TxArgs};

#[tokio::main]
async fn main() -> Result<()> {
    init_logging()?;

    init_env()?;
    tracing::info!("Aprova app init succeeded ...");

    let args = TxArgs::parse();
    tracing::info!("TxArgs: {:?}", args);

    let tx_build_spec = parse_tx_args(&args)?;

    let tx_envelope_wire = build_envelope_wire(tx_build_spec)?;

    let response =send_envelope(tx_envelope_wire).await?;
    tracing::info!("Response: {:?}", response);

    Ok(())
}