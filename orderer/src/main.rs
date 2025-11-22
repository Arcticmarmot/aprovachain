use anyhow::Result;
use clap::{Parser};
use libp2p::identity::{Keypair};
use libp2p::PeerId;
use tokio::{spawn};
use network::handle::*;
use network::p2p::{init_p2p, start_p2p};
use orderer::bootstrap::{init_env, init_logging};
use tokio::sync::mpsc;
use chain::block::Block;
use chain::chain::ChainState;
use chain::mempool::{MempoolHandle};
use consensus::solo::{start_consensus, InputEvent, OutputEvent, SoloService};
use network::handle::P2pCmd::PublishBlock;
use spec::chain::ChainId;
use tx::tx_exec_seal::{TxExecSeal, TxExecSealWire};


pub const TX_COUNT_LIMIT: usize = 3;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct NodeArgs { }

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    init_logging()?;

    // 初始化环境变量
    init_env()?;

    // 解析 NodeArgs
    let args = NodeArgs::parse();

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

    // 共识层初始化
    let local_key = Keypair::ed25519_from_bytes(sk.to_bytes())?;
    let local_id = PeerId::from(local_key.public());

    let (input_tx, input_rx) =
        mpsc::unbounded_channel::<InputEvent>();
    let (output_tx, mut output_rx) =
        mpsc::unbounded_channel::<OutputEvent>();

    let mut block = Block::genesis()?;
    let mut chain_state = ChainState {
        chain_id: ChainId(1000),
        tip_header: block.header
    };
    let mut mempool_handle = MempoolHandle::new();
    let mut solo = SoloService::new(local_id, local_id, chain_state, mempool_handle);

    let tick_input_tx = input_tx.clone();
    spawn(async move {
        start_consensus(&mut solo, tick_input_tx, input_rx, output_tx).await
    });
    
    loop {
        tokio::select! {
            Some(cmd) = event_rx.recv() => {
                match cmd {
                    P2pEvent::ReceivedTx(tx_bytes) => {
                        // 验证字节数组是否是有效交易
                        let wire = TxExecSealWire::try_decode_bcs(&tx_bytes)?;
                        let tx = TxExecSeal::try_from(wire)?;
                        if let Err(err) = input_tx.send(InputEvent::ReceivedTx(tx_bytes)) {
                            tracing::warn!(target:"orderer::event", %err, "input event sending failed")
                        }
                    },
                    P2pEvent::ReceivedBlock(block_bytes) => {
                        tracing::info!(target:"orderer::event", "received block");
                        let block = Block::try_decode_bcs(&block_bytes)?;
                        
                        // 验证区块是否有效
                        // let block = Block::try_decode_bcs(&block_bytes)?;
                    }
                }
            },
            Some(output) = output_rx.recv() => {
                match output {
                    OutputEvent::CommitBlock(block_bytes) => {
                        tracing::info!(target:"orderer::event", "commited block");
                        cmd_handle.publish_block(block_bytes);
                    }
                }
            }
        }
    }
}