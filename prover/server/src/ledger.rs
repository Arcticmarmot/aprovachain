use axum::Json;
use serde::{Deserialize, Serialize};
use account::executor::ExecutorId;
use bench::accounts::{accounts_to_map, load_accounts};
use engine::execute::{build_tx_outcome, pre_exec_tx, verify_envelope_wire};
use ledger::call::{generate_access_set, LedgerCall};
use schedule::dispatch::assign_executor_for_tx;
use tx::attestation::TxAttestation;
use tx::envelope::{TxEnvelope, TxEnvelopeWire};
use tx::intent::{TxIntent, TxPayload, TxScale};
use crate::context::{AppState, SubmitTxResponse};
use crate::error::{ApiResult, ServerError};
use crate::pipeline::resp_from_outcome;
use spec::chain::ChainId;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LedgerRequest {
    pub operation: String,

    #[serde(default)]
    pub customer_id: Option<u64>,

    #[serde(default)]
    pub dest_customer_id: Option<u64>,

    #[serde(default)]
    pub source_customer_id: Option<u64>,

    #[serde(default)]
    pub amount: Option<u64>,
}

fn build_envelope_wire_from_req(req: LedgerRequest) -> TxEnvelopeWire {
    let chain_id = ChainId(1000);
    let accounts = load_accounts();
    let accounts_map = accounts_to_map(chain_id, &accounts);

    let operation = req.operation.as_str();

    let call = match operation {
        "transact_savings" => {
            let from = req.customer_id
                .expect("customer_id is required for transact_savings")
                .to_string();

            let amount = req.amount
                .expect("amount is required for transact_savings");

            LedgerCall::Burn { from, amount }
        }

        "deposit_checking" => {
            let to = req.customer_id
                .expect("customer_id is required for deposit_checking")
                .to_string();

            let amount = req.amount
                .expect("amount is required for deposit_checking");

            LedgerCall::Mint { to, amount }
        }

        "send_payment" => {
            let from = req.source_customer_id
                .expect("source_customer_id is required for send_payment")
                .to_string();

            let to = req.dest_customer_id
                .expect("dest_customer_id is required for send_payment")
                .to_string();

            let amount = req.amount
                .expect("amount is required for send_payment");

            LedgerCall::Transfer { from, to, amount }
        }

        "write_check" => {
            let from = req.customer_id
                .expect("customer_id is required for write_check")
                .to_string();

            let amount = req.amount
                .expect("amount is required for write_check");

            LedgerCall::Burn { from, amount }
        }

        "amalgamate" => {
            let from = req.source_customer_id
                .expect("source_customer_id is required for amalgamate")
                .to_string();

            let to = req.dest_customer_id
                .expect("dest_customer_id is required for amalgamate")
                .to_string();

            // Caliper 原始 amalgamate 没有 amount。
            // 这里先给一个固定金额，用来保留“双账户写冲突”的压力特征。
            let amount = req.amount.unwrap_or(1);

            LedgerCall::Transfer { from, to, amount }
        }

        "query" => {
            let addr = req.customer_id
                .or(req.dest_customer_id)
                .expect("customer_id or dest_customer_id is required for query")
                .to_string();

            LedgerCall::QueryBalance { addr }
        }

        _ => panic!("unsupported ledger operation: {}", operation),
    };

    let ctr_addr_str = String::new();
    let input = call.encode_bcs();
    let access_set = generate_access_set(chain_id, input.clone()).unwrap();

    let payload = TxPayload::Exec {
        ctr_addr_str,
        input,
        access_set,
    };

    let signer_addr = match &call {
        LedgerCall::Transfer { from, .. } => from,
        LedgerCall::Mint { to, .. } => to,
        LedgerCall::Burn { from, .. } => from,
        LedgerCall::QueryBalance { addr } => addr,
    };

    let sk = accounts_map
        .get(signer_addr)
        .expect("signer account not found");

    let scale = TxScale::try_from(18).expect("invalid tx scale");
    let tx_intent = TxIntent::create(chain_id, scale, payload).expect("create tx intent failed");
    let tx_envelope = TxEnvelope::create(tx_intent, sk.clone());

    TxEnvelopeWire::from(&tx_envelope)
}

pub async fn handle_ledger_request(state: AppState, req: LedgerRequest) -> ApiResult<SubmitTxResponse> {
    // 加载状态信息
    let db_handle = state.db_handle;
    let cmd_handle = state.cmd_handle;
    let sk = state.sk;
    let schedule = state.schedule;
    let prove_mode = state.prove_mode;
    let dispatch_config = state.dispatch_config;
    let workload_config = state.server_base_config.workload;
    let self_exec_id = ExecutorId(sk.verifying_key());

    let wire = build_envelope_wire_from_req(req);
    let envelope = verify_envelope_wire(wire)?;
    let send_ts = envelope.intent.timestamp;
    let payload = &envelope.intent.payload;
    match payload {
        TxPayload::Deploy { .. } | TxPayload::Update { .. } => {
            let outcome = build_tx_outcome(&db_handle, envelope, prove_mode)?;
            let response = resp_from_outcome(&outcome)?;
            let tx = TxAttestation::create(outcome, sk);
            let tx_bytes = tx.to_canonical_bytes();
            tracing::info!(target: "executor::event", len=?tx_bytes.len(), "tx_size");
            // 广播交易
            cmd_handle.publish_tx(tx_bytes)?;
            Ok(Json(response))
        }
        TxPayload::Exec { ctr_addr_str, input, access_set } => {
            let envelope_id = envelope.tx_id();
            let exec_id = match assign_executor_for_tx(&db_handle, &envelope_id, send_ts, dispatch_config, workload_config)? {
                Some(exec_id) => { exec_id },
                None => {
                    tracing::info!(target: "node::server", %envelope_id, "no metrics yet, fall back to self as executor");
                    self_exec_id
                }
            };
            tracing::info!(target:"node::server", %exec_id, "executor id");
            if exec_id == self_exec_id {
                tracing::info!(target:"node::server", "handle envelope myself");
                // 交易放入任务队列
                let scale = pre_exec_tx(&db_handle, &envelope, ctr_addr_str, input, access_set)?;
                match db_handle.load_ts_height(send_ts)? {
                    Some(send_height) => {
                        schedule.push(envelope.clone(), scale, send_height).await;
                    }
                    None => { return Err(ServerError::GenesisTs) }
                }
            } else {
                // 广播 envelope 到执行层
                tracing::info!(target:"node::server", "gossip envelope");
                cmd_handle.publish_envelope(envelope.to_canonical_bytes())?;
            }
            // 返回 response
            let ctr_addr_str = ctr_addr_str.clone();
            Ok(Json(SubmitTxResponse::Pending { ctr_addr_str, executor_id: exec_id }))
        }
    }
}
