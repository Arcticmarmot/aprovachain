use anyhow::Result;
use clap::{Parser};
use libp2p::identity::{Keypair};
use libp2p::PeerId;
use tokio::{spawn};
use network::handle::*;
use network::runtime::{init_p2p, run_p2p};
use orderer::bootstrap::{init_env, init_logging};
use tokio::sync::{mpsc, watch};
use chain::chain::ChainState;
use chain::mempool::{MempoolHandle};
use consensus::solo::handle::{SoloCmd, SoloCmdHandle, SoloEvent, SoloEventHandle};
use consensus::solo::service::{start_consensus, SoloService};
use network::behaviour::behaviour::PeerRole;
use orderer::handle::{on_block_commited, on_block_received, on_tx_received};
use spec::chain::ChainId;

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
    let _ = NodeArgs::parse();

    let (p2p_cmd_tx, p2p_cmd_rx) =
        mpsc::unbounded_channel::<P2pCmd>();
    let (p2p_event_tx, mut p2p_event_rx) =
        mpsc::unbounded_channel::<P2pEvent>();
    let p2p_cmd_hdl = P2pCmdHandle::new(p2p_cmd_tx.clone());
    let p2p_event_hdl = P2pEventHandle::new(p2p_event_tx.clone());
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let (sk, peer_set, swarm) = init_p2p(PeerRole::Orderer)?;
    // p2p 接收P2pCmd命令，发出P2pEvent事件
    let p2p_shutdown_rx = shutdown_rx.clone();
    spawn(async move {
        let _ = run_p2p(peer_set, swarm, p2p_cmd_rx, p2p_event_hdl, p2p_shutdown_rx).await;
    });
    tracing::info!(target:"orderer::init", "p2p init success...");

    // 共识层初始化
    let local_key = Keypair::ed25519_from_bytes(sk.to_bytes())?;
    let local_id = PeerId::from(local_key.public());

    let (solo_cmd_tx, solo_cmd_rx) =
        mpsc::unbounded_channel::<SoloCmd>();
    let (solo_event_tx, mut solo_event_rx) =
        mpsc::unbounded_channel::<SoloEvent>();
    
    let chain_state = ChainState {
        chain_id: ChainId(1000),
        tip_header_opt: None
    };
    let mempool_handle = MempoolHandle::new();
    let solo = SoloService::new(local_id, local_id, chain_state, mempool_handle);

    let solo_cmd_hdl = SoloCmdHandle::new(solo_cmd_tx.clone());
    let solo_event_hdl = SoloEventHandle::new(solo_event_tx.clone());
    spawn(async move {
        start_consensus(solo, solo_cmd_rx, solo_cmd_hdl, solo_event_hdl).await
    });
    tracing::info!(target:"orderer::init", "consensus init success...");
    
    let solo_cmd_hdl = SoloCmdHandle::new(solo_cmd_tx);
    loop {
        tokio::select! {
            Some(cmd) = p2p_event_rx.recv() => {
                match cmd {
                    P2pEvent::TxReceived(tx_bytes) => {
                        tracing::info!(target:"orderer::event", "orderer received tx");
                        if let Err(err) = on_tx_received(tx_bytes, &solo_cmd_hdl) {
                            tracing::error!(target:"orderer::event", %err);
                        }
                    },
                    P2pEvent::BlockReceived(block_bytes) => {
                        tracing::info!(target:"orderer::event", "orderer received block");
                        if let Err(err) = on_block_received(block_bytes) {
                            tracing::error!(target:"orderer::event", %err);
                        }
                    }
                    _ => { }
                }
            },
            Some(output) = solo_event_rx.recv() => {
                match output {
                    SoloEvent::BlockCommited { block_bytes } => {
                        tracing::info!(target:"orderer::event", "commited block");
                        if let Err(err) = on_block_commited(block_bytes, &p2p_cmd_hdl) {
                            tracing::error!(target:"orderer::event", %err);
                        }
                    }
                }
            }
        }
    }
}