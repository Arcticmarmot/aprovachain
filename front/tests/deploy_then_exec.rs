use std::collections::BTreeSet;
use std::time::Duration;
use clap::Parser;
use tokio::time::sleep;
use account::address::ContractAddress;
use front::bootstrap::{init_env, init_logging};
use front::handler::{build_envelope_wire, parse_tx_args, parse_tx_args_with, send_envelope, TxArgs};
use server::context::SubmitTxResponse;
use spec::chain::ChainId;
use tx::intent::TxPayload;

#[tokio::test]
pub async fn deploy_then_exec() {
    // Init
    init_logging().unwrap();
    init_env().unwrap();

    // Deploy Contract
    unsafe { std::env::remove_var("DEPLOY_ELF"); }
    unsafe { std::env::remove_var("DEPLOY_JSON"); }
    unsafe { std::env::remove_var("EXEC_JSON"); }
    
    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
        "--payload-type", "Deploy",
        "--deploy-elf", concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/elf/pass.bin"),
    ]).expect("parse args");
    tracing::info!(target:"apps::init", "TxArgs: {:?}", args);

    let tx_build_spec = parse_tx_args(&args).unwrap();

    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();

    let response = send_envelope(tx_envelope_wire).await.unwrap();
    let json = response.json::<SubmitTxResponse>().await.unwrap();
    tracing::info!(target:"apps::resp", "Response: {:?}", json);
    let ctr_addr_str = match json {
        SubmitTxResponse::Deploy { ctr_addr_str, .. } => ctr_addr_str,
        _ => { 
            tracing::error!(target: "front::resp", "contract deploy failed");
            return;
        }
    };

    // Exec Contract
    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
    ]).expect("parse args");

    sleep(Duration::from_secs(20)).await;
    tracing::info!(target:"apps::init", "TxArgs: {:?}", args);
    let payload = TxPayload::Exec {
        ctr_addr_str,
        input: vec![1, 2, 3],
        access_set: BTreeSet::new()
    };
    let tx_build_spec = parse_tx_args_with(&args, |_| Ok(payload)).unwrap();
    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();

    let response = send_envelope(tx_envelope_wire).await.unwrap();
    let json = response.json::<SubmitTxResponse>().await;
    tracing::info!(target:"apps::resp", "Response: {:?}", json);
}