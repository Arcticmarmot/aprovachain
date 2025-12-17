mod common;
mod single;

use std::time::Duration;
use clap::Parser;
use tokio::time::MissedTickBehavior;
use account::address::UserAddress;
use account::keypair::{AccountSigningKey, Keypair};
use front::handler::{build_envelope_wire, create_build_spec_by_sk, parse_tx_args, send_envelope_to, TxArgs};
use ledger::call::{generate_access_set, LedgerCall};
use spec::chain::ChainId;
use tx::intent::{TxPayload};
use common::setup::init_test;
use server::context::SubmitTxResponse;
use crate::common::setup::{extract_ctr_addr, req_by_wire_to, sleep_slot};
use rand::prelude::*;

pub const EXECUTOR_URLS: &[&str] = &[
    // "http://cairo.mining-tuna.ts.net:8888/api/submit-tx",
    // "http://minsk.mining-tuna.ts.net:8888/api/submit-tx",
    // "http://mecca.mining-tuna.ts.net:8888/api/submit-tx",
    "http://smolensk.mining-tuna.ts.net:8888/api/submit-tx",
    // "http://belgrade.mining-tuna.ts.net:8888/api/submit-tx",
];

const TX_NUM: usize = 20;
const USER_NUM: usize = 100;
const PERIOD: Duration = Duration::from_secs(1);
const CHAIN_ID: ChainId = ChainId(1000);
const SCALE: u32 = 18;
const AIRDROP_AMOUNT: u64 = 100000;

#[tokio::test]
pub async fn ledger_test() {
    init_test();

    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
        "--scale", "17",
        "--payload-type", "Deploy",
        "--deploy-elf", concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/elf/ledger_guest.bin"),
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

    let mut users: Vec<AccountSigningKey> = Vec::new();
    for _ in 0..USER_NUM {
        let sk = Keypair::generate().signing_key;
        users.push(sk);
    }

    initial_airdrop(ctr_addr_str, &users).await;

    // let total = TX_NUM;
    // let per = total / EXECUTOR_URLS.len();
    //
    // let mut handles = Vec::new();
    // for url in EXECUTOR_URLS.iter().enumerate() {
    //     let handle = tokio::spawn(send_exec_to_executor(
    //         url.to_string(),
    //         ctr_addr_str.clone(),
    //         per,
    //     ));
    //     handles.push(handle);
    // }
    //
    // for handle in handles {
    //     handle.await.expect("task join").expect("task run");
    // }
}

async fn send_mint(base_url: String, ctr_addr_str: String, sk: &AccountSigningKey) -> anyhow::Result<()> {
    let vk = &sk.verifying_key();
    let addr = UserAddress::from_vk(CHAIN_ID, vk);
    let addr_str = addr.to_bech32m()?;
    let call = LedgerCall::Mint { to: addr_str.clone(), amount: AIRDROP_AMOUNT };

    let input = call.encode_bcs();
    let access_set = generate_access_set(CHAIN_ID, input.clone())?;

    let payload = TxPayload::Exec {
        ctr_addr_str,
        input,
        access_set,
    };
    let tx_scale = rand::random_range(16..=20);

    let spec = create_build_spec_by_sk(CHAIN_ID, &sk, tx_scale, payload)?;
    let wire = build_envelope_wire(spec)?;

    req_by_wire_to(base_url.clone(), wire).await;
    Ok(())
}

async fn initial_airdrop(ctr_addr_str: String, users: &[AccountSigningKey]) {
    for (index, user) in users.iter().enumerate() {
        tracing::info!(target:"apps::resp", %index);
        let url = EXECUTOR_URLS.choose(&mut rand::rng()).unwrap();
        let _ = send_mint(url.to_string(), ctr_addr_str.clone(), user).await;
        tokio::time::sleep(PERIOD).await;
    }
}

async fn send_exec_to_executor(base_url: String, ctr_addr_str: String, sk: AccountSigningKey, tx_num: usize) -> anyhow::Result<()> {
    let mut ticker = tokio::time::interval(PERIOD);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
    let vk = &sk.verifying_key();
    let addr = UserAddress::from_vk(CHAIN_ID, vk);
    let addr_str = addr.to_bech32m()?;
    for _ in 0..tx_num {
        ticker.tick().await;
        let call = LedgerCall::Mint { to: addr_str.clone(), amount: 10 };

        let input = call.encode_bcs();
        let access_set = generate_access_set(CHAIN_ID, input.clone())?;

        let payload = TxPayload::Exec {
            ctr_addr_str: ctr_addr_str.clone(),
            input,
            access_set,
        };

        let spec = create_build_spec_by_sk(CHAIN_ID, &sk, SCALE, payload)?;
        let wire = build_envelope_wire(spec)?;

        req_by_wire_to(base_url.clone(), wire).await;
    }
    Ok(())
}

