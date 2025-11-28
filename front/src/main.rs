use clap::Parser;
use anyhow::{Result};
use front::bootstrap::{init_env, init_logging};
use front::handler::{build_envelope_wire, parse_tx_args, send_envelope, TxArgs};
use server::context::SubmitTxResponse;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    init_logging()?;
    // 初始化环境变量
    init_env()?;
    tracing::info!(target:"apps::init", "Aprova app init succeeded ...");

    // 解析 TxArgs
    let args = TxArgs::parse();
    tracing::info!(target:"apps::init", "TxArgs: {:?}", args);

    // 通过 TxArgs 构建 TxBuildSpec
    let tx_build_spec = parse_tx_args(&args)?;

    // 通过 TxBuildSpec 构建 TxEnvelopeWire
    let tx_envelope_wire = build_envelope_wire(tx_build_spec)?;

    // 构建客户端发送请求
    let response = send_envelope(tx_envelope_wire).await?;
    let json = response.json::<SubmitTxResponse>().await;
    tracing::info!(target:"apps::resp", "Response: {:?}", json);
    Ok(())
}