use axum::{routing::post, Router};
use anyhow::Result;
use std::net::SocketAddr;
use tokio::sync::watch::Receiver;
use network::handle::{P2pCmdHandle};
use account::keypair::AccountSigningKey;
use db::handle::DBHandle;
use platform::config::ProveMode;
use task::schedule::TaskSchedule;
use crate::context::AppState;
use crate::handle::submit_tx;

pub async fn run_server(sk: AccountSigningKey, db_handle: DBHandle,
                        cmd_handle: P2pCmdHandle, schedule: TaskSchedule,
                        shutdown_rx: Receiver<bool>, prove_mode: ProveMode) -> Result<()> {
    let state = AppState {
        db_handle,
        sk,
        cmd_handle,
        schedule,
        prove_mode,
    };
    let node = Router::new()
        .route("/api/submit-tx", post(submit_tx))
        .with_state(state);
    let addr: SocketAddr = "0.0.0.0:8888".parse()?;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8888").await?;
    tracing::info!(target:"node::axum", "node listening on http(s)://{addr} ...");
    axum::serve(listener, node)
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