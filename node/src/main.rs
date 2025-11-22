use axum::{routing::post, Router};
use anyhow::Result;
use std::net::SocketAddr;
use clap::{arg, Parser};
use tokio::{signal, spawn};
use db::runtime::{init_db, close_db, DBFileMode};
use network::handle::{P2pCmd, P2pCmdHandle, P2pEvent, P2pEventHandle};
use network::p2p::{init_p2p, start_p2p};
use node::bootstrap::{init_env, init_logging};
use node::handler::{submit_tx};
use tokio::sync::mpsc;
use account::keypair::AccountSigningKey;
use chain::block::Block;
use db::controller::kv_put;
use node::context::AppState;

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
    tracing::info!(target:"node::db", "rocksdb({db_file_mode:?}) init success...");

    let (cmd_tx, cmd_rx) =
        mpsc::unbounded_channel::<P2pCmd>();
    let (event_tx, mut event_rx) =
        mpsc::unbounded_channel::<P2pEvent>();
    let cmd_handle = P2pCmdHandle::new(cmd_tx.clone());
    let event_handle = P2pEventHandle::new(event_tx.clone());

    let (sk, peer_set, swarm) = init_p2p()?;
    // p2p 接收P2pCmd命令，发出P2pEvent事件
    spawn(async move {
        let _ = start_p2p(peer_set, swarm, cmd_rx, event_handle).await;
    });
    tracing::info!("p2p init success...");

    let _ = init_server(db_file_mode, sk, cmd_handle).await?;


    loop {
        tokio::select! {
            Some(cmd) = event_rx.recv() => {
                match cmd {
                    P2pEvent::ReceivedTx(_) => { },
                    P2pEvent::ReceivedBlock(block_bytes) => {
                        tracing::info!(target:"orderer::event", "received block");
                        let block = Block::try_decode_bcs(&block_bytes)?;
                        kv_put(&block.header.tx_root, &block_bytes)?;
                    }
                }
            },
        }
    }
}

    async fn init_server(db_file_mode: DBFileMode, sk: AccountSigningKey, cmd_handle: P2pCmdHandle) -> Result<()> {
    let state = AppState {
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
        .with_graceful_shutdown(shutdown_signal(db_file_mode))
        .await?;
    Ok(())
}

async fn shutdown_signal(mode: DBFileMode) {
    let _ = signal::ctrl_c().await;
    let _ = close_db(mode);
    eprintln!("shutting down");
}