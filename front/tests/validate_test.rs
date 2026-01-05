mod common;

mod single;

use clap::Parser;
use front::handler::{build_envelope_wire, create_build_spec, parse_tx_args, send_envelope, TxArgs};
use spec::chain::ChainId;
use common::setup::init_test;
use server::context::SubmitTxResponse;
use crate::common::setup::{extract_ctr_addr, req_by_wire, sleep_for};
use apps::ctr_io::AccessSet;
use platform::bench::{bench_csv_begin, bench_csv_path};
use platform::config::load_base_config;
use primitives::trans::u64_to_be_vec;
use tx::intent::TxPayload;

const FIBONACCI_ITERS: &[u64] = &[
    100, 100, 100, 100, 100,
    100, 100, 100, 100, 100,
    100, 100, 100, 100, 100,
    100, 100, 100, 100, 100,
    100, 100, 100, 100, 100,
    100, 100, 100, 100, 100,
    100, 100, 100, 100, 100,
    100, 100, 100, 100, 100,
    100, 100, 100, 100, 100,
    100, 100, 100, 100, 100,
    100, 100, 100, 100, 100,
    100, 100, 100, 100, 100,
];

const GROTH16_TIME: u64 = 30;
const SUCCINCT_TIME: u64 = 30;

#[tokio::test]
pub async fn validate_test() {
    init_test();

    let base = load_base_config();
    let chain_id = ChainId(base.chain_id);
    let slot_secs = base.slot_secs;
    let scale_csv = base.scale_csv;
    let prove_scheme = base.prove.scheme;
    let csv_name = format!("{scale_csv}-{prove_scheme}-validate.csv");
    let csv_path = bench_csv_path(&csv_name);
    bench_csv_begin(&csv_path).unwrap();
    sleep_for(slot_secs + 1).await;

    // deploy fibonacci
    let ctr_addr_str = deploy_fibonacci().await;
    sleep_for(slot_secs + 1).await;

    for (index, &iters) in FIBONACCI_ITERS.iter().enumerate() {
        let scale = 20;
        exec_fibonacci(chain_id, iters, scale, ctr_addr_str.clone()).await;
        sleep_for(1).await;
    }
}

async fn exec_fibonacci(chain_id: ChainId, iters: u64, scale: u32, ctr_addr_str: String) {
    let input = u64_to_be_vec(iters);
    let access_set = AccessSet::new();

    let payload = TxPayload::Exec {
        ctr_addr_str,
        input,
        access_set,
    };

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