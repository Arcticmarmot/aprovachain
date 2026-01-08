mod common;
mod single;

use common::setup::init_test;
use crate::common::setup::req_get_catalogs;

#[tokio::test]
pub async fn get_catalogs_test() {
    init_test();
    let response = req_get_catalogs().await;
    tracing::info!(target:"smallbank", ?response);
}

