use std::time::Duration;
use tokio::time::sleep;
use front::bootstrap::{init_env, init_logging};
use front::handler::{build_envelope_wire, parse_tx_args, send_envelope, send_envelope_to, TxArgs};
use platform::config::load_base_config;
use server::context::SubmitTxResponse;
use tx::envelope::TxEnvelopeWire;

pub const SMOLENSK: &'static str = "main1guwa5cdjvwtc8m86tmrjtkknqtee759k77j7qz";
pub const KOL_SERVER: &'static str = "main16n6z9xz7j5nled2neqsj8qtmcwdnqz6gwsks9f";


pub async fn sleep_a_while() {
    sleep(Duration::from_secs(2)).await
}
pub async fn sleep_slot() {
    let base = load_base_config();
    sleep(Duration::from_secs(base.slot_secs)).await
}

pub fn init_test() {
    // 初始化日志和环境变量
    init_logging().unwrap();
    init_env().unwrap();
    // 排除环境变量干扰
    unsafe { std::env::remove_var("PAYLOAD_TYPE"); }
    unsafe { std::env::remove_var("CTR_ADDR_STR"); }
    unsafe { std::env::remove_var("DEPLOY_ELF"); }
    unsafe { std::env::remove_var("UPDATE_ELF"); }
    unsafe { std::env::remove_var("DEPLOY_JSON"); }
    unsafe { std::env::remove_var("EXEC_JSON"); }
}

pub async fn req_by_args(args: &TxArgs) -> SubmitTxResponse {
    tracing::info!(target:"apps::init", "TxArgs: {:?}", args);

    let tx_build_spec = parse_tx_args(args).unwrap();

    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();

    let resp = send_envelope(tx_envelope_wire).await.unwrap();
    let parsed_resp = resp.json::<SubmitTxResponse>().await.unwrap();
    tracing::info!(target:"apps::resp", "Response: {:?}", parsed_resp);
    parsed_resp
}
pub async fn req_by_args_to(url: String, args: &TxArgs) -> SubmitTxResponse {
    tracing::info!(target:"apps::init", "TxArgs: {:?}", args);

    let tx_build_spec = parse_tx_args(args).unwrap();

    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();

    let resp = send_envelope_to(url, tx_envelope_wire).await.unwrap();
    let parsed_resp = resp.json::<SubmitTxResponse>().await.unwrap();
    tracing::info!(target:"apps::resp", "Response: {:?}", parsed_resp);
    parsed_resp
}

pub async fn req_by_wire(wire: TxEnvelopeWire) -> SubmitTxResponse {
    let resp = send_envelope(wire).await.unwrap();
    let parsed_resp = resp.json::<SubmitTxResponse>().await.unwrap();
    tracing::info!(target:"apps::resp", "Response: {:?}", parsed_resp);
    parsed_resp
}

pub async fn req_by_wire_to(url: String, wire: TxEnvelopeWire) -> SubmitTxResponse {
    let resp = send_envelope_to(url, wire).await.unwrap();
    tracing::info!(target:"apps::resp", "Response: {:?}", resp);
    let parsed_resp = resp.json::<SubmitTxResponse>().await.unwrap();
    tracing::info!(target:"apps::resp", "parsed_resp: {:?}", parsed_resp);
    parsed_resp
}
pub fn extract_ctr_addr(resp: SubmitTxResponse) -> anyhow::Result<String> {
    match resp {
        SubmitTxResponse::Deploy { ctr_addr_str, .. } => Ok(ctr_addr_str),
        SubmitTxResponse::Pending { ctr_addr_str, .. } => Ok(ctr_addr_str),
        other => anyhow::bail!("deploy failed: {:?}", other),
    }
}