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
pub async fn ledger_mint() {
    // Init
    init_logging().unwrap();
    init_env().unwrap();

    // Exec Contract
    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
    ]).expect("parse args");

    sleep(Duration::from_secs(20)).await;
    tracing::info!(target:"apps::init", "TxArgs: {:?}", args);

    let mint = LedgerCall::QueryBalance { addr: SMOLENSK.to_string() };
    let input = mint.encode_bcs();
    let access_set = generate_access_set(ChainId(args.chain_id), input.clone()).unwrap();

    let ctr_addr_str = "mainctr1alvlmpl8ws7cqp257mt8cteny2wwmuw2m3l23k".to_string();
    let payload = TxPayload::Exec {
        ctr_addr_str,
        input,
        access_set
    };

    let tx_build_spec = parse_tx_args_with(&args, |_| Ok(payload)).unwrap();
    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();

    let response = send_envelope(tx_envelope_wire).await.unwrap();
    let json = response.json::<SubmitTxResponse>().await;
    tracing::info!(target:"apps::resp", "Response: {:?}", json);
}