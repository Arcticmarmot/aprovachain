use tx::attestation::TxAttestation;
use crate::pipeline::resp_from_outcome;
use crate::context::{AppState, SubmitTxResponse};
use crate::error::{ApiResult};
use axum::body::{Bytes};
use axum::extract::State;
use axum::Json;
use account::executor::ExecutorId;
use contract::contract::Contract;
use engine::execute::{build_tx_outcome, verify_and_build_envelope};
use schedule::dispatch::assign_executor_for_tx;
use tx::intent::TxPayload;

/// 交易提交处理函数
pub async fn submit_tx(State(state) : State<AppState>, envelope_bytes: Bytes) -> ApiResult<SubmitTxResponse> {
    // 加载状态信息
    let db_handle = state.db_handle;
    let cmd_handle = state.cmd_handle;
    let sk = state.sk;
    let self_exec_id = ExecutorId(sk.verifying_key());

    let envelope = verify_and_build_envelope(envelope_bytes.as_ref())?;
    let tx_envelope_id = envelope.tx_id();

    let exec_id = match assign_executor_for_tx(&db_handle, &tx_envelope_id, envelope.intent.timestamp)? {
        Some(exec_id) => { exec_id },
        None => {
            tracing::info!(target: "node::server", %tx_envelope_id, "no metrics yet, fall back to self as executor");
            self_exec_id
        }
    };
    tracing::info!(target:"node::server", %exec_id, "executor id");

    if exec_id == self_exec_id {
        tracing::info!(target:"node::server", "I'll do it");
        // 执行交易
        let outcome = build_tx_outcome(&db_handle, envelope)?;
        let response = resp_from_outcome(&outcome)?;

        let tx = TxAttestation::create(outcome, sk);
        let tx_bytes = tx.to_canonical_bytes();
        tracing::info!(target: "node::server", len=?tx_bytes.len(), "tx_size");

        // 广播 tx 到共识层
        cmd_handle.publish_tx(tx_bytes)?;

        // 返回 response
        Ok(Json(response))
    } else {
        // 广播 envelope 到执行层
        cmd_handle.publish_envelope(envelope.to_canonical_bytes())?;
        let intent = envelope.intent;
        let payload = intent.payload;
        match payload {
            TxPayload::Deploy { image_id, elf_hash, .. } => {
                let ctr = Contract::create(intent.chain_id, &image_id, &elf_hash,
                                           &envelope.verifying_key, intent.nonce);
                let ctr_addr_str = ctr.addr.to_bech32m()?;
                Ok(Json(SubmitTxResponse::Submitted { ctr_addr_str,executor_id: exec_id }))
            }
            TxPayload::Update { ctr_addr_str, .. } => {
                Ok(Json(SubmitTxResponse::Submitted { ctr_addr_str, executor_id: exec_id }))
            }
            TxPayload::Exec { ctr_addr_str, .. } => {
                Ok(Json(SubmitTxResponse::Submitted { ctr_addr_str, executor_id: exec_id }))
            }
        }

    }
}