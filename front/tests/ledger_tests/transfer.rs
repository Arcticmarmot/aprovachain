use clap::Parser;
use front::bootstrap::{init_env, init_logging};
use front::handler::{build_envelope_wire, parse_tx_args_with, send_envelope, TxArgs};
use ledger::call::{generate_access_set, LedgerCall};
use server::context::SubmitTxResponse;
use spec::chain::ChainId;
use tx::intent::TxPayload;
use crate::ledger_tests::common::{KOL_SERVER, SMOLENSK};

#[tokio::test]
pub async fn ledger_transfer() {
    // Init
    init_logging().unwrap();
    init_env().unwrap();

    // Exec Contract
    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
    ]).expect("parse args");

    tracing::info!(target:"apps::init", "TxArgs: {:?}", args);

    let mint = LedgerCall::Transfer { from: SMOLENSK.to_string(), to: KOL_SERVER.to_string(), amount: 10 };
    let input = mint.encode_bcs();
    let access_set = generate_access_set(ChainId(args.chain_id), input.clone()).unwrap();

    let ctr_addr_str = "mainctr1ljy8eljx08pnxlhzr02ptgekd58zfvufuse4eh".to_string();
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