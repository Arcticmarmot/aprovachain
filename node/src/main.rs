use axum::{routing::post, Router};
use anyhow::Result;
use std::net::SocketAddr;
use clap::{arg, Parser};
use tokio::{signal, spawn};
use db::runtime::{init_db, close_db, DBFileMode};
use network::swarm::{init_p2p, start_p2p};
use node::bootstrap::{init_env, init_logging};
use node::handler::submit_tx;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct NodeArgs {
    #[clap(next_help_heading = "The Chain Id of the Tx")]
    #[arg(short, long, env, value_enum)]
    db_file_mode: DBFileMode,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    init_logging()?;

    // 初始化环境变量
    init_env()?;

    // 解析 NodeArgs
    let args = NodeArgs::parse();

    // 初始化数据库
    let db_file_mode = args.db_file_mode;
    let _ = init_db(db_file_mode)?;
    tracing::info!("Node init success...");

    let (mut peer_set, mut swarm) = init_p2p()?;
    tracing::info!("p2p init success...");
    spawn(async move {
        let _ = start_p2p(&mut peer_set, &mut swarm).await;
    });
    let _ = init_server(db_file_mode).await?;
    Ok(())
}

async fn init_server(db_file_mode: DBFileMode) -> Result<()> {
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