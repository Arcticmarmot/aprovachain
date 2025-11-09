use clap::Parser;
use apps::bootstrap::{init_env, init_logging};
use apps::handler::{build_envelope_wire, parse_tx_args, parse_tx_args_with, send_envelope, TxArgs};
use node::handler::SubmitTxResponse;
use tx::tx_intent::TxPayload;

#[tokio::test]
pub async fn exec_by_json() {
    // Init
    init_logging().unwrap();
    init_env().unwrap();

    // Deploy Contract
    unsafe { std::env::remove_var("DEPLOY_ELF"); }
    unsafe { std::env::remove_var("EXEC_JSON"); }
    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
        "--address", "main1rkz8cf5gp8725xazjlquya90vrs7xrmu4rchec",
        "--verifying-key", "1941e9f4b332ecdfa782dee03751635e06a08bc23633f208586b762b25f009ff",
        "--signing-key", "590afc882c5b674440874414e9edbbc1aff7f0948a311fabbebd1794e4f664a2",
        "--payload-type", "Deploy",
        "--deploy-elf", concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/elf/uav.bin"),
    ]).expect("parse args");
    tracing::info!("TxArgs:{:?}", args);

    let tx_build_spec = parse_tx_args(&args).unwrap();

    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();

    let response =send_envelope(tx_envelope_wire).await.unwrap();
    let json = response.json::<SubmitTxResponse>().await.unwrap();
    tracing::info!("Response: {:?}", json);
    let ctr_addr = match json {
        SubmitTxResponse::Deploy { ctr_addr, .. } => {
            Some(ctr_addr)
        },
        _ => None
    }.unwrap();

    // Exec Contract
    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
        "--address", "main1rkz8cf5gp8725xazjlquya90vrs7xrmu4rchec",
        "--verifying-key", "1941e9f4b332ecdfa782dee03751635e06a08bc23633f208586b762b25f009ff",
        "--signing-key", "590afc882c5b674440874414e9edbbc1aff7f0948a311fabbebd1794e4f664a2",
    ]).expect("parse args");
    tracing::info!("TxArgs:{:?}", args);
    let payload = TxPayload::Exec {
        ctr_addr,
        input: vec![100, 200],
    };
    let tx_build_spec = parse_tx_args_with(&args, |_| Ok(payload)).unwrap();
    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();

    let response =send_envelope(tx_envelope_wire).await.unwrap();
    let json = response.json::<SubmitTxResponse>().await;
    tracing::info!("Response: {:?}", json);
}