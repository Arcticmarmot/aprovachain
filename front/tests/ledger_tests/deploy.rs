use clap::Parser;
use front::handler::{TxArgs};
use crate::ledger_tests::common::{init_test, req_by_args};

#[tokio::test]
pub async fn ledger_deploy() {
    init_test();
    let args = TxArgs::try_parse_from([
        "apps",
        "--chain-id", "1000",
        "--payload-type", "Deploy",
        "--deploy-elf", concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/elf/ledger_guest.bin"),
    ]).expect("parse args");

    req_by_args(&args).await;
}