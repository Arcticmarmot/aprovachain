mod handler;
mod error;

use axum::{
    routing::post,
    Router
};
use anyhow::Result;
use std::net::SocketAddr;
use tokio::signal;
use db::runtime::{init_db, close_db, DBMode};
use crate::handler::submit_tx;
#[tokio::main]
async fn main() -> Result<()> {
    // 初始化数据库
    let _ = init_db(DBMode::Persistent)?;
    println!("rocksdb init success");
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
    let _ = close_db(DBMode::Persistent);
    eprintln!("shutting down");
}