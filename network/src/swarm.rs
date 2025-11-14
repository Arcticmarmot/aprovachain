use std::time::Duration;
use futures::StreamExt;
use libp2p::{noise, tcp, yamux, PeerId, identity};
use libp2p::swarm::{SwarmEvent, Swarm};
use anyhow::Result;
use crate::behaviour::behaviour::{PeerBehaviour, PeerEvent};
use crate::behaviour::peer_set::PeerSet;
use tokio::sync::mpsc;
use crate::cmd::NetworkCmd;

pub fn init_p2p() -> Result<(PeerSet, Swarm<PeerBehaviour>)> {
    let local_key = identity::Keypair::generate_ed25519();
    let local_id = PeerId::from(local_key.public());
    tracing::info!(target:"net::node", peer=%local_id, "node started");

    let disc_behaviour = PeerBehaviour::new(&local_key);

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
        .with_behaviour(|_| disc_behaviour)?
        .with_swarm_config(|cfg| { cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)) })
        .build();
    swarm.listen_on("/ip4/0.0.0.0/tcp/33333".parse()?)?;

    // 创建 PeerSet 记录在线节点
    let peer_set = PeerSet::new(local_id);
    Ok((peer_set, swarm))
}

pub async fn start_p2p( peer_set: &mut PeerSet,  swarm: &mut Swarm<PeerBehaviour>, cmd_rx: &mut mpsc::UnboundedReceiver<NetworkCmd>) {
    let _ = PeerSet::init(swarm);
    loop {
        tokio::select! {
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    NetworkCmd::PublishTx(tx_bytes) => {
                        let _ = swarm.behaviour_mut().publish_tx(tx_bytes);
                    }
                }
            },

            event = swarm.select_next_some() => {
                match event {
                    SwarmEvent::NewListenAddr {address, ..} => {
                        tracing::info!(target:"net::listen", addr=%address, "listening");
                    }
                    SwarmEvent::ConnectionEstablished { peer_id,endpoint ,.. } => {
                        tracing::info!(target:"net::conn", peer=%peer_id, endpoint=?endpoint, "connection established");
                    }
                    SwarmEvent::ConnectionClosed {peer_id,endpoint, ..} => {
                        tracing::info!(target:"net::conn", peer=%peer_id, endpoint=?endpoint, "connection closed");
                    }
                    // DiscoveryEvent 事件处理
                    SwarmEvent::Behaviour(PeerEvent::PeerUp(peer_id, addrs_opt)) => {
                        tracing::info!(target:"net::disc", peer=%peer_id, addrs=?addrs_opt, "peer up");
                        swarm.behaviour_mut().kad_peer_up(&peer_id, addrs_opt.clone());
                        peer_set.on_peer_up(peer_id, addrs_opt);
                        peer_set.refresh(swarm);
                    },
                    SwarmEvent::Behaviour(PeerEvent::PeerDown(peer_id)) => {
                        tracing::info!(target:"net::disc", peer=%peer_id, "peer down");
                        peer_set.on_peer_down(peer_id);
                        peer_set.refresh(swarm);
                    },
                    SwarmEvent::Behaviour(PeerEvent::FoundPeers(peers)) => {
                        tracing::info!(target:"net::disc", known_peers=peers.len(), "mdns discovered candidates");
                        swarm.behaviour_mut().kad_found_peers(&peers);
                        peer_set.on_found_peers(peers);
                        peer_set.refresh(swarm);
                    },
                    SwarmEvent::Behaviour(PeerEvent::TxReceived) => {
                        tracing::info!(target:"net::gossip", "tx received");
                    },
                    _ => {}
                }
            }
        }
    }
}
