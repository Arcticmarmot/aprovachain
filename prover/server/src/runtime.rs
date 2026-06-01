use axum::{routing::post, Router};
use anyhow::Result;
use std::net::SocketAddr;
use axum::routing::get;
use tokio::sync::watch::Receiver;
use network::handle::{P2pCmdHandle};
use account::keypair::AccountSigningKey;
use bench::accounts::load_accounts;
use db::handle::DBHandle;
use platform::config::{BaseConfig, DispatchConfig, ProveMode};
use task::schedule::TaskSchedule;
use crate::context::AppState;
use crate::handle::{get_catalogs, get_stats, submit_smallbank_call, submit_tx};

pub async fn run_server(sk: AccountSigningKey, db_handle: DBHandle,
                        cmd_handle: P2pCmdHandle, schedule: TaskSchedule,
                        shutdown_rx: Receiver<bool>, prove_mode: ProveMode,
                        dispatch_config: DispatchConfig, server_base_config: BaseConfig) -> Result<()> {
    let accounts = load_accounts();
    let state = AppState {
        db_handle,
        sk,
        cmd_handle,
        schedule,
        prove_mode,
        dispatch_config,
        server_base_config,
        accounts
    };
    let router = Router::new()
        .route("/api/submit-tx", post(submit_tx))
        .route("/api/smallbank", post(submit_smallbank_call))
        .route("/api/get-catalogs", post(get_catalogs))
        .route("/api/stats", get(get_stats))
        .with_state(state);
    let addr: SocketAddr = "0.0.0.0:8888".parse()?;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8888").await?;
    tracing::info!(target:"node::axum", "node listening on http(s)://{addr} ...");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal(shutdown_rx))
        .await?;
    Ok(())
}

async fn shutdown_signal(mut shutdown_rx: Receiver<bool>) {
    if *shutdown_rx.borrow() {
        return;
    }
    while shutdown_rx.changed().await.is_ok() {
        if *shutdown_rx.borrow() {
            tracing::info!(target:"server::signal", "shutdown received, stopping server");
            break;
        }
    }
}