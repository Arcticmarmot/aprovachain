use std::time::Duration;
use clap::Parser;
use tokio::time::sleep;
use front::bootstrap::{init_env, init_logging};
use front::handler::{build_envelope_wire, parse_tx_args, parse_tx_args_with, send_envelope, TxArgs};
use ledger::call::{generate_access_set, LedgerCall};
use server::context::SubmitTxResponse;
use spec::chain::ChainId;
use tx::intent::TxPayload;

const SMOLENSK: &'static str = "main1guwa5cdjvwtc8m86tmrjtkknqtee759k77j7qz";
const KOL_SERVER: &'static str = "main16n6z9xz7j5nled2neqsj8qtmcwdnqz6gwsks9f";

#[tokio::test]
pub async fn ledger_deploy_then_exec_dev() {
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
        "--deploy-elf", "/home/woolf/aprova/aprova/target/riscv-guest/methods/ledger_guest/riscv32im-risc0-zkvm-elf/release/ledger_guest.bin",
    ]).expect("parse args");
    tracing::info!(target:"apps::init", "TxArgs: {:?}", args);

    let tx_build_spec = parse_tx_args(&args).unwrap();

    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();

    let response = send_envelope(tx_envelope_wire).await.unwrap();
    let json = response.json::<SubmitTxResponse>().await.unwrap();
    tracing::info!(target:"apps::resp", "Response: {:?}", json);

    let ctr_addr_str = match json {
        SubmitTxResponse::Deploy { ctr_addr_str, .. } => {
            tracing::info!(target: "front::resp", %ctr_addr_str, "ctr addr: ");
            ctr_addr_str
        },
        _ => {
            tracing::error!(target: "front::resp", "contract deploy failed");
            return;
        }
    };

    sleep(Duration::from_secs(16)).await;

    let mint = LedgerCall::Transfer { from: SMOLENSK.to_string(), to: KOL_SERVER.to_string(), amount: 10 };
    let input = mint.encode_bcs();
    let access_set = generate_access_set(ChainId(args.chain_id), input.clone()).unwrap();

    let payload = TxPayload::Exec {
        ctr_addr_str,
        input,
        access_set
    };

    let tx_build_spec = parse_tx_args_with(&args, |_| Ok(payload)).unwrap();
    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();

    let response = send_envelope(tx_envelope_wire).await.unwrap();
    let json = response.json::<SubmitTxResponse>().await.unwrap();
    tracing::info!(target:"apps::resp", "Response: {:?}", json);
}