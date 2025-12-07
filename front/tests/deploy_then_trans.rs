mod common;

use clap::Parser;
use front::handler::{build_envelope_wire, create_build_spec, parse_tx_args, parse_tx_args_with, send_envelope, TxArgs};
use ledger::call::{generate_access_set, LedgerCall};
use server::context::SubmitTxResponse;
use spec::chain::ChainId;
use tx::intent::TxPayload;
use common::setup::{init_test, req_by_args, req_by_wire, sleep_slot, KOL_SERVER, SMOLENSK};

#[tokio::test]
pub async fn ledger_deploy_then_trans() {
    init_test();
    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
        "--scale", "2",
        "--payload-type", "Deploy",
        "--deploy-elf", concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/elf/ledger_guest.bin"),
    ]).expect("parse args");

    let resp = req_by_args(&args).await;

    let ctr_addr_str = match resp {
        SubmitTxResponse::Deploy { ctr_addr_str, .. } => {
            tracing::info!(target: "front::resp", %ctr_addr_str, "ctr addr: ");
            ctr_addr_str
        },
        _ => {
            tracing::error!(target: "front::resp", "contract deploy failed");
            return;
        }
    };

    sleep_slot().await;

    let chain_id = ChainId(args.chain_id);
    let scale = args.scale;
    let mint = LedgerCall::Transfer { from: SMOLENSK.to_string(), to: KOL_SERVER.to_string(), amount: 10 };
    let input = mint.encode_bcs();
    let access_set = generate_access_set(chain_id, input.clone()).unwrap();

    let payload = TxPayload::Exec {
        ctr_addr_str,
        input,
        access_set
    };

    let tx_build_spec = create_build_spec(chain_id, scale, payload).unwrap();
    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();

    req_by_wire(tx_envelope_wire).await;
}