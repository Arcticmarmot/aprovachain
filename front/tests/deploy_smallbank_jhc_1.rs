mod common;

mod single;

use std::collections::HashMap;
use clap::Parser;
use account::keypair::AccountSigningKey;
use front::handler::{build_envelope_wire, create_build_spec_by_sk, parse_tx_args, send_envelope, send_envelope_on_url, TxArgs};
use spec::chain::ChainId;
use common::setup::init_test;
use server::context::SubmitTxResponse;
use crate::common::setup::{extract_ctr_addr, req_by_wire, req_get_catalogs, sleep_for_millis, sleep_for_slot};
use bench::accounts::{accounts_to_map, load_accounts};
use bench::smallbank::{gen_simplified_smallbank, workload_high, workload_low};
use ledger::call::{generate_access_set, LedgerCall};
use platform::config::load_base_config;
use tx::intent::TxPayload;
#[tokio::test]
pub async fn deploy_smallbank_jhc_1() {
    init_test();

    let base = load_base_config();

    let slot_secs = base.slot_secs;

    sleep_for_slot(slot_secs).await;
    let ctr_addr_str = deploy_ledger().await;
    tracing::info!(target:"apps::init", "ctr_addr_str: {:?}", ctr_addr_str);
    sleep_for_slot(slot_secs).await;
}

async fn deploy_ledger() -> String {
    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
        "--scale", "16",
        "--payload-type", "Deploy",
        "--deploy-elf", concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/elf/smallbank_guest.bin"),
    ]).expect("parse args");

    tracing::info!(target:"apps::init", "TxArgs: {:?}", args);

    let tx_build_spec = parse_tx_args(&args).unwrap();

    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();
    let response = send_envelope_on_url(tx_envelope_wire.clone(),
                                        "http://192.168.1.116:8888/api/submit-tx").await.unwrap();
    let parsed_resp = response.json::<SubmitTxResponse>().await.unwrap();
    tracing::info!(target:"apps::resp", "Response: {:?}", parsed_resp);

    let ctr_addr_str = extract_ctr_addr(parsed_resp).unwrap();
    ctr_addr_str
}