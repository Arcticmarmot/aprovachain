mod common;

mod single;

use clap::Parser;
use front::handler::{build_envelope_wire, create_build_spec, parse_tx_args, send_envelope, send_envelope_to, TxArgs};
use spec::chain::ChainId;
use common::setup::init_test;
use server::context::SubmitTxResponse;
use crate::common::setup::{extract_ctr_addr, req_by_wire, sleep_a_while, sleep_slot};
use rand::prelude::*;
use apps::ctr_io::AccessSet;
use platform::config::load_base_config;
use primitives::trans::u64_to_be_vec;
use tx::intent::TxPayload;

#[tokio::test]
pub async fn scale_test() {
    init_test();

    let base = load_base_config();

    let ctr_addr_str = deploy_fibonacci().await;

    sleep_slot().await;

    let input = u64_to_be_vec(100);
    let access_set = AccessSet::new();

    let payload = TxPayload::Exec {
        ctr_addr_str,
        input,
        access_set,
    };

    let chain_id = ChainId(1000);
    let scale = 18;

    let tx_build_spec = create_build_spec(chain_id, scale, payload).unwrap();
    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();

    req_by_wire(tx_envelope_wire).await;
}

async fn deploy_fibonacci() -> String {
    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
        "--scale", "16",
        "--payload-type", "Deploy",
        "--deploy-elf", concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/elf/fibonacci.bin"),
    ]).expect("parse args");

    tracing::info!(target:"apps::init", "TxArgs: {:?}", args);

    let tx_build_spec = parse_tx_args(&args).unwrap();

    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();
    let response = send_envelope(tx_envelope_wire.clone()).await.unwrap();
    let parsed_resp = response.json::<SubmitTxResponse>().await.unwrap();
    tracing::info!(target:"apps::resp", "Response: {:?}", parsed_resp);

    let ctr_addr_str = extract_ctr_addr(parsed_resp).unwrap();
    ctr_addr_str
}