use std::time::Duration;
use libp2p::{identify, ping, mdns, Multiaddr, PeerId, identity, kad, gossipsub, StreamProtocol};
use libp2p::gossipsub::{Message, MessageAuthenticity, MessageId};
use libp2p::kad::store::MemoryStore;
use libp2p::swarm::{NetworkBehaviour};
use crate::behaviour::gossip::GossipTopic;
use crate::error::{Result};

const APROVA_KAD_PROTO: &'static str = "/aprova/kad/v0.1";
const APROVA_GOSSIP_PROTO: &'static str = "/aprova/gossip/v0.1";
const HEART_BEAT_INTERVAL: u64 = 30;
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "PeerEvent")]
pub struct PeerBehaviour {
    pub ping: ping::Behaviour,
    pub identify: identify::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub gossipsub: gossipsub::Behaviour,
}

#[derive(Debug)]
pub enum PeerEvent {
    PeerUp(PeerId, Option<Vec<Multiaddr>>),
    PeerDown(PeerId),
    FoundPeers(Vec<(PeerId, Multiaddr)>),
    TxReceived(PeerId, MessageId, Message),
    BlockReceived,
    Ignore
}

impl From<ping::Event> for PeerEvent {
    fn from(event: ping::Event) -> Self {
        match &event.result {
            Ok(rtt) => {
                tracing::debug!(target:"net::ping", peer=%event.peer, ?rtt, "ping ok");
                PeerEvent::PeerUp(event.peer, None)
            },
            Err(err) => {
                tracing::info!(target:"net::ping", peer=%event.peer, ?err, "ping failed");
                PeerEvent::PeerDown(event.peer)
            }
        }
    }
}

impl From<identify::Event> for PeerEvent {
    fn from(event: identify::Event) -> Self {
        match event {
            identify::Event::Received {peer_id, info, ..} => {
                tracing::info!(target:"net::identify", %peer_id, "identify received");
                tracing::debug!(target:"net::identify",
                    %peer_id, agent=%info.agent_version, listen_addrs=?info.listen_addrs, protocols=?info.protocols, observed_addr=%info.observed_addr,
                    "identify details");
                let addrs = info.listen_addrs;
                PeerEvent::PeerUp(peer_id, Some(addrs)) // 传递对端自报的地址
            },
            identify::Event::Pushed {peer_id, info, ..} => {
                tracing::info!(target:"net::identify", %peer_id, "identify received");
                tracing::debug!(target:"net::identify",
                    %peer_id, agent=%info.agent_version, listen_addrs=?info.listen_addrs, protocols=?info.protocols, observed_addr=%info.observed_addr,
                    "identify details");
                let addrs = info.listen_addrs;
                PeerEvent::PeerUp(peer_id, Some(addrs)) // 传递对端自报的地址
            },
            identify::Event::Sent { .. } => {
                PeerEvent::Ignore
            },
            identify::Event::Error {peer_id, error, ..} => {
                tracing::warn!(target:"net::identify", %peer_id, ?error, "identify error");
                PeerEvent::PeerDown(peer_id)
            },
        }
    }
}

impl From<mdns::Event> for PeerEvent {
    fn from(event: mdns::Event) -> Self {
        match event {
            mdns::Event::Discovered(peers) => {
                tracing::info!(target:"network::mdns", count=peers.len(), "mdns discovered peers");
                tracing::debug!(target:"network::mdns", ?peers, "mdns discovered details");
                PeerEvent::FoundPeers(peers.into_iter().collect())
            },
            mdns::Event::Expired(peers) => {
                tracing::info!(target:"network::mdns", count=peers.len(), "mdns expired peers");
                tracing::trace!(target:"network::mdns", ?peers, "mdns expired details");
                PeerEvent::Ignore
            }
        }
    }
}

impl From<kad::Event> for PeerEvent {
    fn from(event: kad::Event) -> Self {
        use kad::Event::*;
        match event {
            RoutablePeer { peer, address } => {
                tracing::info!(target:"network::kad", peer=%peer, addr=%address, "kad routable peer");
                PeerEvent::FoundPeers(vec![(peer, address)])
            },
            PendingRoutablePeer { peer, address } => {
                tracing::info!(target:"network::kad", peer=%peer, addr=%address, "kad pending routable peer");
                PeerEvent::FoundPeers(vec![(peer, address)])
            },
            UnroutablePeer { peer } => {
                tracing::info!(target:"network::kad", peer=%peer, "kad unroutable peer");
                PeerEvent::PeerDown(peer)
            },
            InboundRequest { request } => {
                tracing::debug!(target:"network::kad", ?request, "kad request");
                PeerEvent::Ignore
            },
            OutboundQueryProgressed {id, result, stats, step} => {
                tracing::debug!(target: "network::kad", %id, ?result, ?stats, ?step);
                PeerEvent::Ignore
            }
            ModeChanged { new_mode } => {
                tracing::debug!(target:"network::kad", %new_mode, "kad mode changed");
                PeerEvent::Ignore
            }
            RoutingUpdated {peer, is_new_peer, addresses, .. } => {
                tracing::debug!(target:"network::kad", %peer, %is_new_peer, ?addresses, "kad routing update");
                PeerEvent::Ignore
            }
        }

    }
}

impl From<gossipsub::Event> for PeerEvent {
    fn from(event: gossipsub::Event) -> Self {
        use gossipsub::Event::*;
        match event {
            Message { message, message_id, propagation_source } => {
                tracing::info!(target:"network::gossip", source=%propagation_source, message=?message, message_id=%message_id, "message comes");
                PeerEvent::TxReceived(propagation_source, message_id, message)
            },
            Subscribed {peer_id, topic} => {
                tracing::info!(target:"network::gossip", %peer_id, %topic, "subscribe");
                PeerEvent::Ignore
            },
            Unsubscribed {peer_id, topic} => {
                tracing::info!(target:"network::gossip", %peer_id, %topic, "unsubscibe");
                PeerEvent::Ignore
            },
            GossipsubNotSupported {peer_id} => {
                tracing::info!(target:"network::gossip", %peer_id, "unsupported");
                PeerEvent::Ignore
            },
            SlowPeer {peer_id, failed_messages} => {
                tracing::info!(target:"network::gossip", %peer_id, ?failed_messages, "unsupported");
                PeerEvent::Ignore
            }
        }
    }
}

impl PeerBehaviour {
    pub fn new(local_key: &identity::Keypair) -> Self {
        let public = local_key.public();
        let local_peer_id = PeerId::from(public.clone());

        let ping = ping::Behaviour::new(
            ping::Config::new().with_interval(Duration::from_secs(HEART_BEAT_INTERVAL))
        );
        let identify_cfg = identify::Config::new("/aprova/v0.1".into(), public.clone());
        let identify = identify::Behaviour::new(identify_cfg);

        let mdns_cfg = mdns::Config::default();
        let mdns = mdns::tokio::Behaviour::new(
            mdns_cfg, PeerId::from(&public.clone()))
            .expect("mdns create failed");

        let store = MemoryStore::new(local_peer_id);
        let kad_cfg = kad::Config::new(StreamProtocol::new(APROVA_KAD_PROTO));
        let mut kademlia = kad::Behaviour::with_config(local_peer_id, store, kad_cfg);
        kademlia.set_mode(Some(kad::Mode::Server));

        let gossipsub_cfg = gossipsub::ConfigBuilder::default()
            .protocol_id_prefix(APROVA_GOSSIP_PROTO)
            .validation_mode(gossipsub::ValidationMode::Strict)
            .max_transmit_size(5 * 1024 * 1024)
            .build().expect("build gossipsub config");
        let mut gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_cfg
        ).expect("gossipsub build");

        let tx_topic = GossipTopic::Tx.ident();
        let block_topic = GossipTopic::Block.ident();
        gossipsub.subscribe(&tx_topic).expect("subscribe tx");
        gossipsub.subscribe(&block_topic).expect("subscribe block");

        Self {
            ping,
            identify,
            mdns,
            kademlia,
            gossipsub
        }
    }

    pub fn publish_tx(&mut self, tx_bytes: Vec<u8>) -> Result<()> {
        tracing::info!(target:"network::gossip", len=%tx_bytes.len(), "tx size: ");
        let topic = GossipTopic::Tx.ident();
        self.gossipsub.publish(topic, tx_bytes)?;
        Ok(())
    }

    pub fn publish_block(&mut self, block_bytes: Vec<u8>) -> Result<()> {
        tracing::info!(target:"network::gossip", len=%block_bytes.len(), "block size: ");
        let topic = GossipTopic::Block.ident();
        let _ = self.gossipsub.publish(topic, block_bytes);
        Ok(())
    }

    pub fn kad_mut(&mut self) -> &mut kad::Behaviour<MemoryStore> {
        &mut self.kademlia
    }

    pub fn kad_peer_up(&mut self, peer_id: &PeerId, addrs: Option<Vec<Multiaddr>>) {
        if addrs.is_none() { return; }
        for addr in addrs.unwrap() {
            tracing::info!(target:"network::kad", %peer_id, ?addr, "kad add addr");
            self.kad_mut().add_address(peer_id, addr);
        }
    }

    pub fn kad_found_peers(&mut self, peers: &Vec<(PeerId, Multiaddr)>) {
        for (peer_id, addr) in peers {
            tracing::info!(target:"network::kad", %peer_id, ?addr, "kad add addr");
            self.kad_mut().add_address(peer_id, addr.clone());
        }
    }
}
