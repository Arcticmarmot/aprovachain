use anyhow::Result;
use clap::{arg, Parser};
use tokio::{spawn};
use network::handle::{P2pCmd, P2pHandle, TxCmd};
use network::consensus_p2p::{init_p2p, start_p2p};
use consensus::bootstrap::{init_env, init_logging};
use tokio::sync::mpsc;
use tx::tx_exec_seal::{TxExecSeal, TxExecSealWire};

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
    let (tx_sender, mut tx_receiver) =
        mpsc::unbounded_channel::<TxCmd>();

    let (sk, peer_set, swarm) = init_p2p()?;
    tracing::info!("p2p init success...");
    spawn(async move {
        let _ = start_p2p(peer_set, swarm, cmd_receiver, tx_sender).await;
    });
    let p2p_handle = P2pHandle::new(cmd_sender.clone());
    let mut tx_pool: Vec<TxExecSeal> = Vec::new();
    // 2. 等待退出信号 或 p2p 任务异常结束
    loop {
        tokio::select! {
            Some(cmd) = tx_receiver.recv() => {
                match cmd {
                    TxCmd::PushTx(tx_bytes) => {
                        let wire = TxExecSealWire::try_decode_bcs(&tx_bytes)?;
                        let tx = TxExecSeal::try_from(wire)?;
                        tx_pool.push(tx);
                        tracing::info!(target:"consensus::main", len=tx_pool.len())
                    }
                }
            },
        }
    }
}