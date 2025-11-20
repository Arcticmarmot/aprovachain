use anyhow::Result;
use clap::{arg, Parser};
use tokio::{spawn};
use network::handle::{P2pCmd, P2pHandle};
use network::p2p::{init_p2p, start_p2p};
use consensus::bootstrap::{init_env, init_logging};
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct NodeArgs {
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    init_logging()?;

    // 初始化环境变量
    init_env()?;

    // 解析 NodeArgs
    let args = NodeArgs::parse();

    let (cmd_sender, cmd_receiver) =
        mpsc::unbounded_channel::<P2pCmd>();
    let (sk, peer_set, swarm) = init_p2p()?;
    tracing::info!("p2p init success...");
    spawn(async move {
        let _ = start_p2p(peer_set, swarm, cmd_receiver).await;
    });
    let p2p_handle = P2pHandle::new(cmd_sender.clone());

    // 2. 等待退出信号 或 p2p 任务异常结束
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("ctrl-c received, shutting down...");
        }
    }
    Ok(())
}