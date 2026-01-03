use tx::attestation::TxAttestation;
use crate::pipeline::resp_from_outcome;
use crate::context::{AppState, SubmitTxResponse};
use crate::error::{ApiResult, ServerError};
use axum::body::{Bytes};
use axum::extract::State;
use axum::Json;
use account::executor::ExecutorId;
use engine::execute::{build_tx_outcome, pre_exec_tx, verify_and_build_envelope};
use schedule::dispatch::assign_executor_for_tx;
use tx::intent::TxPayload;

/// 交易提交处理函数
pub async fn submit_tx(State(state) : State<AppState>, envelope_bytes: Bytes) -> ApiResult<SubmitTxResponse> {
    // 加载状态信息
    let db_handle = state.db_handle;
    let cmd_handle = state.cmd_handle;
    let sk = state.sk;
    let schedule = state.schedule;
    let prove_mode = state.prove_mode;
    let self_exec_id = ExecutorId(sk.verifying_key());

    let envelope = verify_and_build_envelope(envelope_bytes.as_ref())?;
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
            let exec_id = match assign_executor_for_tx(&db_handle, &envelope_id, send_ts)? {
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

