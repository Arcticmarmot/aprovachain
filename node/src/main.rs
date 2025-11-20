use axum::{routing::post, Router};
use anyhow::Result;
use std::net::SocketAddr;
use clap::{arg, Parser};
use tokio::{signal, spawn};
use db::runtime::{init_db, close_db, DBFileMode};
use network::handle::{P2pCmd, P2pHandle};
use network::p2p::{init_p2p, start_p2p};
use node::bootstrap::{init_env, init_logging};
use node::handler::{submit_tx, AppState};
use tokio::sync::mpsc;
use account::keypair::AccountSigningKey;

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
    tracing::info!("rocksdb({db_file_mode:?}) init success...");

    let (cmd_sender, cmd_receiver) =
        mpsc::unbounded_channel::<P2pCmd>();
    let (sk, peer_set, swarm) = init_p2p()?;
    tracing::info!("p2p init success...");
    spawn(async move {
        let _ = start_p2p(peer_set, swarm, cmd_receiver).await;
    });
    let p2p_handle = P2pHandle::new(cmd_sender.clone());
    let _ = init_server(db_file_mode, sk, p2p_handle).await?;
    Ok(())
}

async fn init_server(db_file_mode: DBFileMode, sk: AccountSigningKey, p2p_handle: P2pHandle) -> Result<()> {
    let state = AppState {
        sk,
        p2p_handle,
    };
    let node = Router::new()
        .route("/api/submit-tx", post(submit_tx))
        .with_state(state);
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