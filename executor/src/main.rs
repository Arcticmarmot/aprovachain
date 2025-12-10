use anyhow::Result;
use std::process::exit;
use clap::{arg, Parser};
use tokio::{signal, spawn};
use db::runtime::{init_db, close_db, DBFileMode};
use network::handle::{P2pCmd, P2pCmdHandle, P2pEvent, P2pEventHandle};
use network::runtime::{init_p2p, run_p2p};
use executor::bootstrap::{init_env, init_logging};
use tokio::sync::mpsc;
use db::handle::DBHandle;
use network::behaviour::behaviour::PeerRole;
use executor::handle::{handle_block_received, handle_envelope_received};
use server::runtime::run_server;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct NodeArgs {
    #[clap(next_help_heading = "database file mode")]
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
    let db_handle = DBHandle::new()?;
    tracing::info!(target:"executor::init", "rocksdb({db_file_mode:?}) init success...");

    let (p2p_cmd_tx, p2p_cmd_rx) =
        mpsc::unbounded_channel::<P2pCmd>();
    let (p2p_event_tx, mut p2p_event_rx) =
        mpsc::unbounded_channel::<P2pEvent>();
    let p2p_cmd_hdl = P2pCmdHandle::new(p2p_cmd_tx.clone());
    let p2p_event_hdl = P2pEventHandle::new(p2p_event_tx.clone());

    let (sk, peer_set, swarm) = init_p2p(PeerRole::Executor)?;
    // p2p 接收P2pCmd命令，发出P2pEvent事件
    spawn(async move {
        let _ = run_p2p(peer_set, swarm, p2p_cmd_rx, p2p_event_hdl).await;
    });
    tracing::info!(target:"executor::init", "p2p init success...");

    let server_sk= sk.clone();
    let server_db_handle = db_handle.clone();
    // 开启 http 服务
    spawn(async move {
        let _ = run_server(server_sk, server_db_handle, p2p_cmd_hdl).await;
    });
    tracing::info!(target:"executor::init", "server init success...");
    
    loop {
        tokio::select! {
            Some(cmd) = p2p_event_rx.recv() => {
                match cmd {
                    P2pEvent::EnvelopeReceived(envelope_bytes) => {
                        tracing::info!(target:"executor::event", "executor received envelope");
                        let p2p_cmd_hdl = P2pCmdHandle::new(p2p_cmd_tx.clone());
                        if let Err(err) = handle_envelope_received(p2p_cmd_hdl, sk.clone(), envelope_bytes) {
                            tracing::warn!(target:"executor::event", %err);
                        }
                    }
                    P2pEvent::BlockReceived(block_bytes) => {
                        tracing::info!(target:"executor::event", "executor received block");
                        if let Err(err) = handle_block_received(&db_handle, block_bytes) {
                            tracing::warn!(target:"executor::event::block", %err);
                        }
                    },
                    _ => { }
                }
            },

            _ = signal::ctrl_c() => {
                tracing::info!(target:"executor::signal", "ctrl-c received, shutting down");
                let _ = close_db(db_file_mode);
                exit(0);
            }
        }
    }
}
