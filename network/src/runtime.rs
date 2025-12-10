use std::fs;
use std::time::Duration;
use libp2p::{noise, tcp, yamux, PeerId};
use libp2p::swarm::{SwarmEvent, Swarm};
use anyhow::Result;
use libp2p::futures::StreamExt;
use libp2p::identity::Keypair;
use crate::behaviour::behaviour::{PeerBehaviour, PeerRole};
use crate::behaviour::peer_set::PeerSet;
use tokio::sync::mpsc::{UnboundedReceiver};
use crate::handle::{P2pCmd, P2pEventHandle};
use account::keypair::{AccountSigningKey, AccountSigningKeyBytes};
use platform::file::{load_node_sk_path};
use crate::behaviour::event::PeerEvent;
use crate::behaviour::gossip::GossipTopic;

pub fn load_node_sk_bytes() -> Result<AccountSigningKeyBytes> {
    let sk_path = load_node_sk_path();
    let sk_hex = fs::read(sk_path)?;
    let mut sk_bytes: AccountSigningKeyBytes = [0u8; 32];
    hex::decode_to_slice(sk_hex, &mut sk_bytes)?;
    Ok(sk_bytes)
}

pub fn init_p2p(role: PeerRole) -> Result<(AccountSigningKey, PeerSet, Swarm<PeerBehaviour>)> {
    // 从文件加载 sk
    let sk_bytes = load_node_sk_bytes()?;
    let local_key = Keypair::ed25519_from_bytes(sk_bytes)?;
    let local_id = PeerId::from(local_key.public());
    let sk = AccountSigningKey::from_bytes(&sk_bytes);

    let disc_behaviour = PeerBehaviour::new(&local_key, role);

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
        .with_behaviour(|_| disc_behaviour)?
        .with_swarm_config(|cfg| { cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)) })
        .build();
    swarm.listen_on("/ip4/0.0.0.0/tcp/33333".parse()?)?;

    // 创建 PeerSet 记录在线节点
    let peer_set = PeerSet::new(local_id);
    Ok((sk, peer_set, swarm))
}

pub async fn run_p2p(
    mut peer_set: PeerSet,
    mut swarm: Swarm<PeerBehaviour>,
    mut cmd_rx: UnboundedReceiver<P2pCmd>,
    event_handle: P2pEventHandle,
) {
    let swarm = &mut swarm;
    let _ = PeerSet::init(swarm);
    loop {
        tokio::select! {
            Some(cmd) = cmd_rx.recv() => {
                match cmd {
                    P2pCmd::PublishEnvelope(envelope_bytes) => {
                        if let Err(err) = swarm.behaviour_mut().publish_envelope(envelope_bytes) {
                            tracing::warn!(target: "net::cmd", %err, "publish envelope cmd")
                        }
                    }
                    P2pCmd::PublishTx(tx_bytes) => {
                        if let Err(err) = swarm.behaviour_mut().publish_tx(tx_bytes) {
                            tracing::warn!(target: "net::cmd", %err, "publish tx cmd")
                        }
                    }
                    P2pCmd::PublishBlock(block_bytes) => {
                        if let Err(err) = swarm.behaviour_mut().publish_block(block_bytes) {
                            tracing::warn!(target: "net::cmd", %err, "publish block cmd")
                        }
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
                        tracing::debug!(target:"net::disc", peer=%peer_id, addrs=?addrs_opt, "peer up");
                        swarm.behaviour_mut().kad_peer_up(&peer_id, addrs_opt.clone());
                        peer_set.on_peer_up(peer_id, addrs_opt);
                        peer_set.refresh(swarm);
                    },
                    SwarmEvent::Behaviour(PeerEvent::PeerDown(peer_id)) => {
                        tracing::info!(target:"net::disc", peer=%peer_id, "peer down");
                        peer_set.refresh(swarm);
                    },
                    SwarmEvent::Behaviour(PeerEvent::FoundPeers(peers)) => {
                        tracing::info!(target:"net::disc", known_peers=peers.len(), "mdns discovered candidates");
                        swarm.behaviour_mut().kad_found_peers(&peers);
                        peer_set.on_found_peers(peers);
                        peer_set.refresh(swarm);
                    },
                    SwarmEvent::Behaviour(PeerEvent::MessageReceived(peer_id, message_id, message)) => {
                        let topic = message.topic;
                        let bytes = message.data;
                        match GossipTopic::from_hash(&topic) {
                            Some(GossipTopic::Envelope) => {
                                if let Err(err) = event_handle.received_envelope(bytes) {
                                    tracing::info!(target: "net::event", %peer_id, %message_id, %err, "receive tx envelope")
                                }
                            }
                            Some(GossipTopic::Tx) => {
                                if let Err(err) = event_handle.received_tx(bytes) {
                                    tracing::info!(target: "net::event", %peer_id, %message_id, %err, "receive tx event")
                                }
                            },
                            Some(GossipTopic::Block) => {
                                if let Err(err) = event_handle.received_block(bytes) {
                                    tracing::info!(target: "net::event", %err, "receive block event")
                                }
                            },
                            None => {
                                tracing::error!(target: "net::event", "bad event")
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}
