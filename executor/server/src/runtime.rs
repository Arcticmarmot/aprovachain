use axum::{routing::post, Router};
use anyhow::Result;
use std::net::SocketAddr;
use tokio::{signal};
use network::handle::{P2pCmdHandle};
use account::keypair::AccountSigningKey;
use db::handle::DBHandle;
use crate::context::AppState;
use crate::handle::submit_tx;

pub async fn run_server(sk: AccountSigningKey, db_handle: DBHandle, cmd_handle: P2pCmdHandle) -> Result<()> {
    let state = AppState {
        db_handle,
        sk,
        cmd_handle,
    };
    let node = Router::new()
        .route("/api/submit-tx", post(submit_tx))
        .with_state(state);
    let addr: SocketAddr = "0.0.0.0:8888".parse()?;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8888").await?;
    tracing::info!(target:"node::axum", "node listening on http(s)://{addr} ...");
    axum::serve(listener, node)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
}