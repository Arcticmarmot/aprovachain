mod handler;
mod error;

use axum::{
    routing::post,
    Router
};
use anyhow::Result;
use std::net::SocketAddr;
use tokio::signal;
use db::init::db_init;
use crate::handler::submit_tx;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化数据库
    db_init();
    let node = Router::new().route("/api/submit-tx", post(submit_tx));
    let addr: SocketAddr = "0.0.0.0:8888".parse()?;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8888").await?;
    println!("node listening on http://{addr} ...");
    axum::serve(listener, node)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let _ = signal::ctrl_c().await;
    eprintln!("shutting down");
}