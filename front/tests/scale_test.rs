mod common;

mod single;

use clap::Parser;
use front::handler::{build_envelope_wire, create_build_spec, parse_tx_args, send_envelope_to, TxArgs};
use spec::chain::ChainId;
use common::setup::init_test;
use server::context::SubmitTxResponse;
use crate::common::setup::{extract_ctr_addr, req_by_wire, sleep_a_while, sleep_slot};
use rand::prelude::*;
use apps::ctr_io::AccessSet;
use platform::config::load_base_config;
use primitives::trans::u64_to_be_vec;
use tx::intent::TxPayload;

pub const EXECUTOR_URLS: &[&str] = &[
    // "http://cairo.mining-tuna.ts.net:8888/api/submit-tx",
    // "http://minsk.mining-tuna.ts.net:8888/api/submit-tx",
    // "http://mecca.mining-tuna.ts.net:8888/api/submit-tx",
    "http://smolensk.mining-tuna.ts.net:8888/api/submit-tx",
    // "http://belgrade.mining-tuna.ts.net:8888/api/submit-tx",
];

const TX_NUM: usize = 20;
const USER_NUM: usize = 100;
const CHAIN_ID: ChainId = ChainId(1000);
const SCALE: u32 = 18;

#[tokio::test]
pub async fn scale_test() {
    init_test();
    sleep_a_while().await;
    let base = load_base_config();

    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
        "--scale", "17",
        "--payload-type", "Deploy",
        "--deploy-elf", concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/elf/fibonacci.bin"),
    ]).expect("parse args");

    tracing::info!(target:"apps::init", "TxArgs: {:?}", args);

    let tx_build_spec = parse_tx_args(&args).unwrap();

    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();
    let deploy_url = EXECUTOR_URLS.choose(&mut rand::rng()).unwrap();
    let response = send_envelope_to(deploy_url.to_string(), tx_envelope_wire.clone()).await.unwrap();
    let parsed_resp = response.json::<SubmitTxResponse>().await.unwrap();
    tracing::info!(target:"apps::resp", "Response: {:?}", parsed_resp);

    let ctr_addr_str = extract_ctr_addr(parsed_resp).unwrap();

    sleep_slot().await;

    let input = u64_to_be_vec(100000);
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