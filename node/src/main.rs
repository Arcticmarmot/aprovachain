mod handler;
mod error;
mod bootstrap;

use axum::{
    routing::post,
    Router
};
use anyhow::Result;
use std::net::SocketAddr;
use clap::{arg, Parser};
use tokio::signal;
use db::runtime::{init_db, close_db, DBFileMode};
use crate::bootstrap::{init_env, init_logging};
use crate::handler::submit_tx;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct NodeArgs {
    #[clap(next_help_heading = "The Chain Id of the Tx")]
    #[arg(short, long, env, value_enum)]
    db_file_mode: DBFileMode,
}


#[tokio::main]
async fn main() -> Result<()> {
    init_logging()?;

    init_env()?;

    let args = NodeArgs::parse();

    let db_file_mode = args.db_file_mode;

    // 初始化数据库
    let _ = init_db(db_file_mode)?;
    tracing::info!("rocksdb init success...");
    let node = Router::new().route("/api/submit-tx", post(submit_tx));
    let addr: SocketAddr = "0.0.0.0:8888".parse()?;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8888").await?;
    tracing::info!("node listening on http(s)://{addr} ...");
    axum::serve(listener, node)
        .with_graceful_shutdown(shutdown_signal(db_file_mode))
        .await?;
    Ok(())
}

async fn shutdown_signal(mode: DBFileMode) {
    let _ = signal::ctrl_c().await;
    let _ = close_db(mode);
    eprintln!("shutting down");
}