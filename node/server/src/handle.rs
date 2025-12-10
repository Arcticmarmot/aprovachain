use tx::envelope::{TxEnvelopeWire, TxEnvelope};
use tx::attestation::TxAttestation;
use crate::pipeline::{build_tx_outcome, resp_from_outcome};
use crate::context::{AppState, SubmitTxResponse};
use crate::error::{ApiResult};
use axum::body::{Bytes};
use axum::extract::State;
use axum::Json;
use account::executor::ExecutorId;
use schedule::dispatch::assign_executor_for_tx;

/// 交易提交处理函数
pub async fn submit_tx(State(state) : State<AppState>, tx_bytes: Bytes) -> ApiResult<SubmitTxResponse> {
    // 加载状态信息
    let db_handle = state.db_handle;
    let cmd_handle = state.cmd_handle;
    let sk = state.sk;
    let self_exec_id = ExecutorId(sk.verifying_key());

    // 从字节数组构造 TxEnvelope
    let wire: TxEnvelopeWire = TxEnvelopeWire::try_decode_bcs(tx_bytes.as_ref())?;
    let envelope = TxEnvelope::try_from(wire)?;
    let tx_envelope_id = envelope.tx_id();
    // TODO: 重放交易攻击，拒绝重复的 nonce
    // 验证交易签名是否有效
    envelope.self_verify()?;
    tracing::info!(target:"node::server", ?tx_envelope_id, "tx envelope id");

    let exec_id = match assign_executor_for_tx(&db_handle, &tx_envelope_id)? {
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

        // 广播交易
        cmd_handle.publish_tx(tx_bytes)?;

        // 返回 response
        Ok(Json(response))
    } else {
        cmd_handle.publish_envelope(envelope.to_canonical_bytes())?;
        Ok(Json(SubmitTxResponse::Submitted { executor_id: exec_id }))
    }
}