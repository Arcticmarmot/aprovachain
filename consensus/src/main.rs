use anyhow::Result;
use clap::{Parser};
use tokio::{spawn};
use network::handle::*;
use network::p2p::{init_p2p, start_p2p};
use consensus::bootstrap::{init_env, init_logging};
use tokio::sync::mpsc;
use chain::block::Block;
use chain::mempool::{pack_block, Mempool};
use tx::tx_exec_seal::{TxExecSeal, TxExecSealWire};


pub const TX_COUNT_LIMIT: usize = 3;

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
    let mut mempool = Mempool::new();

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

    loop {
        tokio::select! {
            Some(cmd) = event_rx.recv() => {
                match cmd {
                    P2pEvent::ReceivedTx(tx_bytes) => {
                        // 验证字节数组是否是有效交易
                        let wire = TxExecSealWire::try_decode_bcs(&tx_bytes)?;
                        let tx = TxExecSeal::try_from(wire)?;
                        mempool.push_tx(tx);
                        if mempool.count() > TX_COUNT_LIMIT {
                            let new_block = pack_block(&mempool, &block.header, TX_COUNT_LIMIT)?;
                            cmd_handle.publish_block(new_block.to_canonical_bytes())?;
                            block = new_block;
                            tracing::info!(target:"consensus::block", ?block, "published")
                        }
                        tracing::info!(target:"consensus::block", ?block)
                    },
                    P2pEvent::ReceivedBlock(block_bytes) => {

                    }
                }
            },
        }
    }
}