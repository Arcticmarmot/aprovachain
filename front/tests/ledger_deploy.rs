use clap::Parser;
use front::bootstrap::{init_env, init_logging};
use front::handler::{build_envelope_wire, parse_tx_args, send_envelope, TxArgs};
use server::context::SubmitTxResponse;
#[tokio::test]
pub async fn ledger_deploy() {
    // Init
    init_logging().unwrap();
    init_env().unwrap();

    // Deploy Contract
    unsafe { std::env::remove_var("DEPLOY_ELF"); }
    unsafe { std::env::remove_var("DEPLOY_JSON"); }
    unsafe { std::env::remove_var("EXEC_JSON"); }
    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
        "--payload-type", "Deploy",
        "--deploy-elf", concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/elf/ledger_guest.bin"),
    ]).expect("parse args");
    tracing::info!(target:"apps::init", "TxArgs: {:?}", args);

    let tx_build_spec = parse_tx_args(&args).unwrap();

    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();

    let response = send_envelope(tx_envelope_wire).await.unwrap();
    let json = response.json::<SubmitTxResponse>().await.unwrap();
    tracing::info!(target:"apps::resp", "Response: {:?}", json);
    match json {
        SubmitTxResponse::Deploy { ctr_addr_str, .. } => {
            tracing::info!(target: "front::resp", %ctr_addr_str, "ctr addr: ");
        },
        _ => {}
    };
}