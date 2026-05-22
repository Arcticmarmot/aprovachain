use serde::{Deserialize, Serialize};

use account::address::UserAddress;
use account::keypair::AccountSigningKey;
use bench::accounts::load_accounts;
use smallbank::call::{generate_access_set, SmallbankCall};
use spec::chain::ChainId;
use tx::envelope::{TxEnvelope, TxEnvelopeWire};
use tx::intent::{TxIntent, TxPayload, TxScale};

const SCALE: u32 = 18;
const CHAIN_ID: u64 = 1000;
const SMALLBANK_CTR_ADDR: &str = "mainctr17apdmw246n0xsajpdh5jrdmaghamq0fjjtws3c";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SmallbankRequest {
    pub operation: String,

    #[serde(default)]
    pub customer_id: Option<u64>,

    #[serde(default)]
    pub dest_customer_id: Option<u64>,

    #[serde(default)]
    pub source_customer_id: Option<u64>,

    #[serde(default)]
    pub amount: Option<i64>,
}

fn account_sk(accounts: &[AccountSigningKey], customer_id: u64) -> &AccountSigningKey {
    accounts.get(customer_id as usize).expect("account")
}

fn account_addr(chain_id: ChainId, accounts: &[AccountSigningKey], customer_id: u64) -> String {
    let vk = account_sk(accounts, customer_id).verifying_key();
    UserAddress::from_vk(chain_id, &vk).to_bech32m().expect("addr")
}

pub fn build_envelope_wire_from_req(req: SmallbankRequest) -> TxEnvelopeWire {
    let chain_id = ChainId(CHAIN_ID);
    let accounts = load_accounts();

    let signer_id = match req.operation.as_str() {
        "transact_savings" | "deposit_checking" | "write_check" | "query" => {
            req.customer_id.expect("cid")
        }
        "send_payment" | "amalgamate" => {
            req.source_customer_id.expect("src")
        }
        op => panic!("op: {}", op),
    };

    let call = match req.operation.as_str() {
        "transact_savings" => SmallbankCall::TransactSavings {
            customer_id: account_addr(chain_id, &accounts, req.customer_id.expect("cid")),
            amount: req.amount.expect("amt"),
        },

        "deposit_checking" => SmallbankCall::DepositChecking {
            customer_id: account_addr(chain_id, &accounts, req.customer_id.expect("cid")),
            amount: req.amount.expect("amt"),
        },

        "send_payment" => SmallbankCall::SendPayment {
            source_customer_id: account_addr(chain_id, &accounts, req.source_customer_id.expect("src")),
            dest_customer_id: account_addr(chain_id, &accounts, req.dest_customer_id.expect("dst")),
            amount: req.amount.expect("amt"),
        },

        "write_check" => SmallbankCall::WriteCheck {
            customer_id: account_addr(chain_id, &accounts, req.customer_id.expect("cid")),
            amount: req.amount.expect("amt"),
        },

        "amalgamate" => SmallbankCall::Amalgamate {
            source_customer_id: account_addr(chain_id, &accounts, req.source_customer_id.expect("src")),
            dest_customer_id: account_addr(chain_id, &accounts, req.dest_customer_id.expect("dst")),
        },

        "query" => SmallbankCall::Query {
            customer_id: account_addr(chain_id, &accounts, req.customer_id.expect("cid")),
        },

        op => panic!("op: {}", op),
    };

    let input = call.encode_bcs();
    let access_set = generate_access_set(chain_id, input.clone()).expect("access");

    let payload = TxPayload::Exec {
        ctr_addr_str: SMALLBANK_CTR_ADDR.to_string(),
        input,
        access_set,
    };

    let scale = TxScale::try_from(SCALE).expect("scale");
    let tx_intent = TxIntent::create(chain_id, scale, payload).expect("intent");
    let tx_envelope = TxEnvelope::create(tx_intent, account_sk(&accounts, signer_id).clone());

    TxEnvelopeWire::from(&tx_envelope)
}