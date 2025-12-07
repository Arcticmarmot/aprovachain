mod common;

use front::handler::{build_envelope_wire, create_build_spec};
use ledger::call::{generate_access_set, LedgerCall};
use spec::chain::ChainId;
use tx::intent::TxPayload;
use common::setup::{init_test, req_by_wire, SMOLENSK};

#[tokio::test]
pub async fn ledger_mint() {
    init_test();
    let chain_id = ChainId(1000);
    let scale = 2;
    let mint = LedgerCall::Mint { to: SMOLENSK.to_string(), amount: 50 };
    let input = mint.encode_bcs();
    let access_set = generate_access_set(chain_id, input.clone()).unwrap();

    let ctr_addr_str = "mainctr1nnhwm33wvepxsnrkqtqckkt4w7m4js92qk703p".to_string();
    let payload = TxPayload::Exec {
        ctr_addr_str,
        input,
        access_set
    };

    let tx_build_spec = create_build_spec(chain_id, scale, payload).unwrap();
    let tx_envelope_wire = build_envelope_wire(tx_build_spec).unwrap();

    req_by_wire(tx_envelope_wire).await;
}