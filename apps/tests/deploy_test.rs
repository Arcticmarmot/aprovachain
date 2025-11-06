use apps::handler::*;
use apps::bootstrap::*;
use clap::Parser;
// CHAIN_ID=1000
// ADDRESS=main1rkz8cf5gp8725xazjlquya90vrs7xrmu4rchec
// VERIFYING_KEY=1941e9f4b332ecdfa782dee03751635e06a08bc23633f208586b762b25f009ff
// SIGNING_KEY=590afc882c5b674440874414e9edbbc1aff7f0948a311fabbebd1794e4f664a2
// PAYLOAD_TYPE=deploy
// DEPLOY_JSON=/home/woolf/aprova/aprova/apps/fixtures/tx/deploy.json
// DEPLOY_ELF=/home/woolf/aprova/aprova/apps/fixtures/elf/aprova-guest.bin
// EXEC_JSON=/home/woolf/aprova/aprova/apps/fixtures/tx/exec.json

#[tokio::test]
async fn deploy_by_json() {
    init_logging().unwrap();
    init_env().unwrap();
    unsafe { std::env::remove_var("DEPLOY_ELF"); }
    unsafe { std::env::remove_var("EXEC_JSON"); }
    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "2000",
        "--address", "main1rkz8cf5gp8725xazjlquya90vrs7xrmu4rchec",
        "--verifying-key", "1941e9f4b332ecdfa782dee03751635e06a08bc23633f208586b762b25f009ff",
        "--signing-key", "590afc882c5b674440874414e9edbbc1aff7f0948a311fabbebd1794e4f664a2",
        "--payload-type", "deploy",
        "--deploy-json", concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/tx/deploy.json"),
    ]).expect("parse args");
    tracing::info!("TxArgs:{:?}", args);

    let tx_build_spec = parse_tx_args(&args).unwrap();

    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();

    let response =send_envelope(tx_envelope_wire).await.unwrap();
    tracing::info!("Response: {:?}", response);
}