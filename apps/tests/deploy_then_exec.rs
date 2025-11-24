use std::time::Duration;
use clap::Parser;
use tokio::time::sleep;
use apps::bootstrap::{init_env, init_logging};
use apps::handler::{build_envelope_wire, parse_tx_args, parse_tx_args_with, send_envelope, TxArgs};
use server::context::SubmitTxResponse;
use tx::intent::TxPayload;

#[tokio::test]
pub async fn deploy_then_exec() {
    // Init
    init_logging().unwrap();
    init_env().unwrap();

    // Deploy Contract
    unsafe { std::env::remove_var("DEPLOY_ELF"); }
    unsafe { std::env::remove_var("EXEC_JSON"); }
    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
        "--payload-type", "Deploy",
        "--deploy-elf", concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/elf/uav.bin"),
    ]).expect("parse args");
    tracing::info!(target:"apps::init", "TxArgs: {:?}", args);

    let tx_build_spec = parse_tx_args(&args).unwrap();

    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();

    let response =send_envelope(tx_envelope_wire).await.unwrap();
    let json = response.json::<SubmitTxResponse>().await.unwrap();
    tracing::info!(target:"apps::resp", "Response: {:?}", json);
    let ctr_addr_bytes = match json {
        SubmitTxResponse::Deploy { ctr_addr_bytes, .. } => {
            Some(ctr_addr_bytes)
        },
        _ => None
    }.unwrap();

    // Exec Contract
    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
    ]).expect("parse args");

    sleep(Duration::from_secs(20)).await;
    tracing::info!(target:"apps::init", "TxArgs: {:?}", args);
    let payload = TxPayload::Exec {
        ctr_addr_bytes,
        input: vec![100, 200],
    };
    let tx_build_spec = parse_tx_args_with(&args, |_| Ok(payload)).unwrap();
    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();

    let response =send_envelope(tx_envelope_wire).await.unwrap();
    let json = response.json::<SubmitTxResponse>().await;
    tracing::info!(target:"apps::resp", "Response: {:?}", json);
}