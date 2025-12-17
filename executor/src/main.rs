use anyhow::Result;
use std::process::exit;
use clap::{arg, Parser};
use tokio::{signal, spawn};
use db::runtime::{init_db, close_db, DBFileMode};
use network::handle::{P2pCmd, P2pCmdHandle, P2pEvent, P2pEventHandle};
use network::runtime::{init_p2p, run_p2p};
use executor::bootstrap::{init_env, init_logging};
use tokio::sync::{mpsc, watch};
use account::executor::ExecutorId;
use db::handle::DBHandle;
use network::behaviour::behaviour::PeerRole;
use executor::handle::{on_block_received, on_envelope_received};
use server::runtime::run_server;
use task::queue::TaskQueue;
use task::runtime::run_task;

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

    // 新建 envelope 任务队列
    let queue = TaskQueue::new_smallest_first();

    let (p2p_cmd_tx, p2p_cmd_rx) =
        mpsc::unbounded_channel::<P2pCmd>();
    let (p2p_event_tx, mut p2p_event_rx) =
        mpsc::unbounded_channel::<P2pEvent>();
    // 结束信号
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let p2p_cmd_hdl = P2pCmdHandle::new(p2p_cmd_tx.clone());
    let p2p_event_hdl = P2pEventHandle::new(p2p_event_tx.clone());

    let (sk, peer_set, swarm) = init_p2p(PeerRole::Executor)?;
    // p2p 接收P2pCmd命令，发出P2pEvent事件
    let p2p_shutdown_rx = shutdown_rx.clone();
    let p2p_handle =  spawn(async move {
        run_p2p(peer_set, swarm, p2p_cmd_rx, p2p_event_hdl, p2p_shutdown_rx).await;
    });
    tracing::info!(target:"executor::init", "p2p init success...");

    let self_exec_id = ExecutorId(sk.verifying_key());
    tracing::info!(target:"executor::init", %self_exec_id, "self executor id");

    let task_db_handle = db_handle.clone();
    let task_sk= sk.clone();
    let task_queue = queue.clone();
    let task_shutdown_rx = shutdown_rx.clone();
    let task_handle = spawn(async move {
        run_task(task_db_handle, p2p_cmd_hdl, task_sk, task_queue, task_shutdown_rx).await;
    });

    let server_db_handle = db_handle.clone();
    let server_sk= sk.clone();
    let p2p_cmd_hdl = P2pCmdHandle::new(p2p_cmd_tx.clone());
    let server_shutdown_rx = shutdown_rx.clone();
    let server_queue = queue.clone();
    // 开启 http 服务
    let server_handle = spawn(async move {
        let _ = run_server(server_sk, server_db_handle, p2p_cmd_hdl,
                           server_queue, server_shutdown_rx).await;
    });
    tracing::info!(target:"executor::init", "server init success...");

    loop {
        tokio::select! {
            Some(cmd) = p2p_event_rx.recv() => {
                match cmd {
                    P2pEvent::EnvelopeReceived(envelope_bytes) => {
                        tracing::info!(target:"executor::event", "executor received envelope");
                        if let Err(err) = on_envelope_received(queue.clone(), self_exec_id, envelope_bytes).await {
                            tracing::warn!(target:"executor::event", %err);
                        }
                    }
                    P2pEvent::BlockReceived(block_bytes) => {
                        tracing::info!(target:"executor::event", "executor received block");
                        if let Err(err) = on_block_received(&db_handle, block_bytes) {
                            tracing::warn!(target:"executor::event::block", %err);
                        }
                    },
                    _ => { }
                }
            },

            _ = signal::ctrl_c() => {
                tracing::info!(target:"executor::signal", "ctrl-c received, shutting down");
                let _ = shutdown_tx.send(true);
                let _ = task_handle.await;
                let _ = p2p_handle.await;
                let _ = server_handle.await;
                let _ = close_db(db_file_mode);
                exit(0);
            }
        }
    }

}
