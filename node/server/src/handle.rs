use tx::envelope::{TxEnvelopeWire, TxEnvelope};
use axum::body::{Bytes};
use axum::extract::State;
use axum::Json;
use schedule::dispatch::{assign_executor_for_tx, compute_scores_by_window, select_executor_for_tx, WINDOW_SIZE};
use schedule::error::ScheduleError;
use crate::error::{ApiResult};
use tx::attestation::TxAttestation;
use tx::id::TxEnvelopeId;
use crate::context::{AppState, SubmitTxResponse};
use crate::pipeline::{build_tx_outcome, resp_from_outcome};

/// 交易提交处理函数
pub async fn submit_tx(State(state) : State<AppState>, tx_bytes: Bytes) -> ApiResult<SubmitTxResponse> {
    // 加载状态信息
    let db_handle = state.db_handle;
    let cmd_handle = state.cmd_handle;
    let sk = state.sk;
    // 从字节数组构造 TxEnvelope
    let wire: TxEnvelopeWire = TxEnvelopeWire::try_decode_bcs(tx_bytes.as_ref())?;
    let envelope = TxEnvelope::try_from(wire)?;
    let tx_id = envelope.tx_id();
    tracing::info!(target:"node::server", ?tx_id, "tx id");
    let stats_window = db_handle.load_stats_window(WINDOW_SIZE)?;
    let scores = compute_scores_by_window(&stats_window);
    match select_executor_for_tx(&tx_id, &scores) {
        Some(exec_id) => {
            tracing::info!(target:"node::server", ?exec_id, "executor id");
        }
        None => {
            tracing::info!(target:"node::server", "first executor id");
        }
    };

    // TODO: 重放交易攻击，拒绝重复的 nonce
    // 验证交易签名是否有效
    envelope.self_verify()?;
    tracing::info!(target:"node::server", tx_envelope_id=%envelope.tx_id(), "tx envelope id");

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
}