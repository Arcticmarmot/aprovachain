mod common;

use std::time::Duration;
use clap::Parser;
use tokio::time::MissedTickBehavior;
use front::handler::{build_envelope_wire, create_build_spec, TxArgs};
use ledger::call::{generate_access_set, LedgerCall};
use spec::chain::ChainId;
use tx::intent::{TxPayload};
use common::setup::{init_test, SMOLENSK, KOL_SERVER};
use crate::common::setup::{extract_ctr_addr, req_by_args_to, req_by_wire_to, sleep_slot};

pub const EXECUTOR_URLS: &[&str] = &[
    "http://100.82.28.52:8888/api/submit-tx",
    "http://100.94.178.96:8888/api/submit-tx",
    "http://100.107.181.54:8888/api/submit-tx",
];
const TX_NUM: usize = 100;
const PERIOD: Duration = Duration::from_secs(10);
const CHAIN_ID: ChainId = ChainId(1000);
const SCALE: u32 = 17;
async fn send_exec_to_executor(base_url: String, ctr_addr_str: String, tx_num: usize) -> anyhow::Result<()> {
    let mut ticker = tokio::time::interval(PERIOD);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    for i in 0..tx_num {
        ticker.tick().await;
        let call = if i % 2 == 0 {
            let amount = 1_00000;
            LedgerCall::Mint { to: SMOLENSK.to_string(), amount }
        } else {
            let amount = 1;
            LedgerCall::Transfer {
                from: SMOLENSK.to_string(),
                to: KOL_SERVER.to_string(),
                amount,
            }
        };

        let input = call.encode_bcs();
        let access_set = generate_access_set(CHAIN_ID, input.clone())?;

        let payload = TxPayload::Exec {
            ctr_addr_str: ctr_addr_str.clone(),
            input,
            access_set,
        };

        let spec = create_build_spec(CHAIN_ID, SCALE, payload)?;
        let wire = build_envelope_wire(spec)?;

        req_by_wire_to(base_url.clone(), wire).await;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
pub async fn deploy_then_send() {
    init_test();
    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
        "--scale", "17",
        "--payload-type", "Deploy",
        "--deploy-elf", concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/elf/ledger_guest.bin"),
    ]).expect("parse args");

    let mut resp= None;
    for url in EXECUTOR_URLS {
        resp = Some(req_by_args_to(url.to_string(), &args).await);
    }

    let ctr_addr_str = extract_ctr_addr(resp.unwrap()).unwrap();

    sleep_slot().await;

    let total = TX_NUM;
    let per = total / EXECUTOR_URLS.len();

    let mut handles = Vec::new();
    for (idx, url) in EXECUTOR_URLS.iter().enumerate() {
        let handle = tokio::spawn(send_exec_to_executor(
            url.to_string(),
            ctr_addr_str.clone(),
            per,
        ));
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("task join").expect("task run");
    }
}