use anyhow::Result;
use clap::{Parser};
use tokio::{spawn};
use network::handle::*;
use network::p2p::{init_p2p, start_p2p};
use consensus::bootstrap::{init_env, init_logging};
use tokio::sync::mpsc;
use chain::block::Block;
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
    
    let mut block = Block::genesis()?;

    let (cmd_tx, cmd_rx) =
        mpsc::unbounded_channel::<P2pCmd>();
    let (event_tx, mut event_rx) =
        mpsc::unbounded_channel::<P2pEvent>();
    let cmd_handle = P2pCmdHandle::new(cmd_tx.clone());

    let (sk, peer_set, swarm) = init_p2p()?;
    // p2p 接收P2pCmd命令，发出P2pEvent事件
    spawn(async move {
        let _ = start_p2p(peer_set, swarm, cmd_rx, event_tx).await;
    });
    tracing::info!("p2p init success...");

    loop {
        tokio::select! {
            Some(cmd) = event_rx.recv() => {
                match cmd {
                    P2pEvent::PushTx(tx_bytes) => {
                        let wire = TxExecSealWire::try_decode_bcs(&tx_bytes)?;
                        let tx = TxExecSeal::try_from(wire)?;
                        block.push_tx(tx)?;
                        tracing::info!(target:"consensus::main", ?block)
                    }
                }
            },
        }
    }
}