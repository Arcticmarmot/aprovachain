mod common;

mod single;

use std::collections::HashMap;
use clap::Parser;
use account::keypair::AccountSigningKey;
use front::handler::{build_envelope_wire, create_build_spec_by_sk, parse_tx_args, send_envelope, TxArgs};
use spec::chain::ChainId;
use common::setup::init_test;
use server::context::SubmitTxResponse;
use crate::common::setup::{extract_ctr_addr, req_by_wire, req_get_catalogs, sleep_for, sleep_for_millis, sleep_for_slot, try_req_by_wire_to};
use bench::accounts::{accounts_to_map, load_accounts};
use bench::smallbank::{gen_simplified_smallbank, workload_high, workload_low};
use ledger::call::{generate_access_set, LedgerCall};
use platform::config::load_base_config;
use tx::intent::TxPayload;
#[tokio::test]
pub async fn smallbank_test() {
    init_test();

    let base = load_base_config();
    let chain_id = ChainId(base.chain_id);
    let slot_secs = base.slot_secs;

    let accounts = load_accounts();
    let accounts_map = accounts_to_map(chain_id, &accounts);
    let user_addr_strings: Vec<String> = accounts_map.keys().cloned().collect();

    let smallbank = gen_simplified_smallbank(&user_addr_strings, &workload_high(), 1024u64);
    tracing::info!(target: "smallbank", len=?smallbank.len());

    sleep_for_slot(slot_secs).await;
    let ctr_addr_str = deploy_ledger().await;
    sleep_for_slot(slot_secs).await;

    for (index, call) in smallbank.iter().enumerate() {
        tracing::info!(target:"apps::resp", %index, ?call);
        send_call(call, chain_id,
                  ctr_addr_str.clone(), &accounts_map).await;
        sleep_for_millis(50).await;
    }

    sleep_for_slot(slot_secs * 2).await;
    let response = req_get_catalogs().await;
    tracing::info!(target:"smallbank", ?response);
}

async fn send_call(call: &LedgerCall, chain_id: ChainId,
                        ctr_addr_str: String, accounts_map: &HashMap<String, AccountSigningKey>) {
    let input = call.encode_bcs();
    let access_set = generate_access_set(chain_id, input.clone()).unwrap();

    let payload = TxPayload::Exec {
        ctr_addr_str: ctr_addr_str.clone(),
        input,
        access_set,
    };
    let sk = match call {
        LedgerCall::Transfer { from, .. } => {
            accounts_map.get(from).unwrap()
        }
        LedgerCall::Mint { to, .. } => {
            accounts_map.get(to).unwrap()
        }
        LedgerCall::Burn { from,  .. } => {
            accounts_map.get(from).unwrap()
        }
        LedgerCall::QueryBalance { addr } => {
            accounts_map.get(addr).unwrap()
        }
    };
    let tx_scale = 17;
    let spec = create_build_spec_by_sk(chain_id, &sk, tx_scale, payload).expect("build spec failed");
    let wire = build_envelope_wire(spec).expect("build envelope failed");

    req_by_wire(wire).await;
}

async fn deploy_ledger() -> String {
    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
        "--scale", "16",
        "--payload-type", "Deploy",
        "--deploy-elf", concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/elf/ledger_guest.bin"),
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