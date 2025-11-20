use apps::handler::*;
use apps::bootstrap::*;
use clap::Parser;
use node::handler::SubmitTxResponse;

#[tokio::test]
pub async fn exec_by_json() {
    init_logging().unwrap();
    init_env().unwrap();
    unsafe { std::env::remove_var("DEPLOY_ELF"); }
    unsafe { std::env::remove_var("EXEC_JSON"); }
    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
        "--payload-type", "Exec",
        "--exec-json", concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/tx/exec.json"),
    ]).expect("parse args");
    tracing::info!(target:"apps::init", "TxArgs: {:?}", args);

    let tx_build_spec = parse_tx_args(&args).unwrap();

    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();

    let response =send_envelope(tx_envelope_wire).await.unwrap();
    let json = response.json::<SubmitTxResponse>().await;
    tracing::info!(target:"apps::resp", "Response: {:?}", json);
}