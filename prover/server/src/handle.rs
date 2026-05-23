use std::collections::{HashMap};
use tx::attestation::TxAttestation;
use crate::pipeline::resp_from_outcome;
use crate::context::{AppState, SubmitTxResponse};
use crate::error::{ApiResult, ServerError};
use axum::body::{Bytes};
use axum::extract::State;
use axum::Json;
use account::executor::ExecutorId;
use engine::execute::{build_tx_outcome, pre_exec_tx, verify_and_build_envelope, verify_envelope_wire};
use platform::bench::{bench_smallbank_csv_append, bench_smallbank_csv_begin, bench_smallbank_csv_path};
use schedule::dispatch::assign_executor_for_tx;
use tx::intent::TxPayload;
use crate::smallbank::{build_envelope_wire_from_req, SmallbankRequest};

/// 交易提交处理函数
pub async fn submit_tx(State(state): State<AppState>, envelope_bytes: Bytes) -> ApiResult<SubmitTxResponse> {
    // 加载状态信息
    let db_handle = state.db_handle;
    let cmd_handle = state.cmd_handle;
    let sk = state.sk;
    let schedule = state.schedule;
    let prove_mode = state.prove_mode;
    let dispatch_config = state.dispatch_config;
    let workload_config = state.server_base_config.workload;
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

pub async fn submit_smallbank_call(State(state): State<AppState>, Json(req): Json<SmallbankRequest>) -> ApiResult<SubmitTxResponse> {
    // 加载状态信息
    let db_handle = state.db_handle;
    let cmd_handle = state.cmd_handle;
    let sk = state.sk;
    let schedule = state.schedule;
    let prove_mode = state.prove_mode;
    let dispatch_config = state.dispatch_config;
    let workload_config = state.server_base_config.workload;
    let accounts = state.accounts;
    let self_exec_id = ExecutorId(sk.verifying_key());

    let wire = build_envelope_wire_from_req(req, &accounts);
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

pub async fn get_catalogs(State(state): State<AppState>, _: Bytes) -> ApiResult<SubmitTxResponse> {
    tracing::info!(target:"node::server", "get catalogs");
    let db_handle = state.db_handle;

    let ts = platform::clock::unix_time_millis()?;
    let catalogs = db_handle.load_stats_window(usize::MAX, ts)?;
    // tracing::info!(target:"node::server", ?catalogs);
    let mut stats = HashMap::new();
    for catalog in &catalogs {
        for (_, (_, code)) in catalog.iter() {
            let entry: &mut usize = stats.entry(code).or_default();
            *entry += 1;
        }
    }
    let mut stats_by_prover = HashMap::new();
    for catalog in &catalogs {
        for (_, (exec_id, code)) in catalog.iter() {
            let entry: &mut usize = stats_by_prover.entry((exec_id, code)).or_default();
            *entry += 1;
        }
    }
    tracing::info!(target:"node::server", ?stats);
    tracing::info!(target:"node::server", ?stats_by_prover);
    let dispatch_mode = state.dispatch_config.mode;
    let base = state.server_base_config;
    let cons_protocol = base.consensus.protocol;
    let discipline = base.queue.discipline;
    let workload = base.workload;
    let workload_set = workload.workload_set;
    let with_slot = workload.with_slow;
    let test_name = workload.test_name;
    let slot_secs = base.slot_secs;
    let ema_k = base.dispatch.ema_k;
    let tx_num = workload.tx_num;
    let load_multi = workload.load_multi;
    let tps = workload.tps;
    let hetero = workload.heterogeneous;
    let tail = workload.tail;
    let catalog_stats = bench_smallbank_csv_path(
        &format!("catalog-stats-{hetero}-{test_name}-{slot_secs}slot-{dispatch_mode}-{cons_protocol}-{workload_set}-{load_multi}-{discipline}-{with_slot}-{ema_k}K-{tx_num}-{tps}-{tail}"));
    bench_smallbank_csv_begin(&catalog_stats).expect("catalog stats csv begin");
    for (code, count) in stats {
        bench_smallbank_csv_append(&catalog_stats, "total".to_string(), code.to_string(), count)
            .expect("catalog stats csv append");
    }
    for ((prover_id, code), count) in &stats_by_prover {
        bench_smallbank_csv_append(&catalog_stats, prover_id.to_string(), code.to_string(), *count)
            .expect("catalog stats csv append");
    }

    let resp = Json(SubmitTxResponse::CatalogStore);
    Ok(resp)
}


