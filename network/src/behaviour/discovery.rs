use std::time::Duration;
use libp2p::{identify, ping, mdns, Multiaddr, PeerId, identity, kad, gossipsub, StreamProtocol};
use libp2p::gossipsub::{MessageAuthenticity};
use libp2p::kad::store::MemoryStore;
use libp2p::swarm::{NetworkBehaviour};

const APROVA_KAD_PROTO: &'static str = "/aprova/kad/1.0.0";

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "DiscoveryEvent")]
pub struct DiscoveryBehaviour {
    pub ping: ping::Behaviour,
    pub identify: identify::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub gossipsub: gossipsub::Behaviour,
}

#[derive(Debug)]
pub enum DiscoveryEvent {
    PeerUp(PeerId, Option<Vec<Multiaddr>>),
    PeerDown(PeerId),
    FoundPeers(Vec<(PeerId, Multiaddr)>),
    TxReceived,
    BlockReceived,
    Ignore
}

impl From<ping::Event> for DiscoveryEvent {
    fn from(event: ping::Event) -> Self {
        match &event.result {
            Ok(rtt) => {
                tracing::info!(target:"net::ping", peer=%event.peer, ?rtt, "ping ok");
                DiscoveryEvent::PeerUp(event.peer, None)
            },
            Err(err) => {
                tracing::info!(target:"net::ping", peer=%event.peer, ?err, "ping failed");
                DiscoveryEvent::PeerDown(event.peer)
            }
        }
    }
}

impl From<identify::Event> for DiscoveryEvent {
    fn from(event: identify::Event) -> Self {
        match event {
            identify::Event::Received {peer_id, info, ..} => {
                tracing::info!(target:"net::identify", %peer_id, "identify received");
                tracing::debug!(target:"net::identify",
                    %peer_id, agent=%info.agent_version, listen_addrs=?info.listen_addrs, protocols=?info.protocols, observed_addr=%info.observed_addr,
                    "identify details");
                // 传递对端自报的地址
                let addrs = info.listen_addrs;
                DiscoveryEvent::PeerUp(peer_id, Some(addrs))
            },
            identify::Event::Pushed {peer_id, info, ..} => {
                tracing::info!(target:"net::identify", %peer_id, "identify received");
                tracing::debug!(target:"net::identify",
                    %peer_id, agent=%info.agent_version, listen_addrs=?info.listen_addrs, protocols=?info.protocols, observed_addr=%info.observed_addr,
                    "identify details");
                let addrs = info.listen_addrs;
                // 传递对端自报的地址
                DiscoveryEvent::PeerUp(peer_id, Some(addrs))
            },
            identify::Event::Sent { .. } => {
                DiscoveryEvent::Ignore
            },
            identify::Event::Error {peer_id, error, ..} => {
                tracing::warn!(target:"net::identify", %peer_id, ?error, "identify error");
                DiscoveryEvent::PeerDown(peer_id)
            },
        }
    }
}

impl From<mdns::Event> for DiscoveryEvent {
    fn from(event: mdns::Event) -> Self {
        match event {
            mdns::Event::Discovered(peers) => {
                tracing::info!(target:"network::mdns", count=peers.len(), "mdns discovered peers");
                tracing::debug!(target:"network::mdns", ?peers, "mdns discovered details");
                DiscoveryEvent::FoundPeers(peers.into_iter().collect())
            },
            mdns::Event::Expired(peers) => {
                tracing::info!(target:"network::mdns", count=peers.len(), "mdns expired peers");
                tracing::trace!(target:"network::mdns", ?peers, "mdns expired details");
                DiscoveryEvent::Ignore
            }
        }
    }
}

impl From<kad::Event> for DiscoveryEvent {
    fn from(event: kad::Event) -> Self {
        use kad::Event::*;
        match event {
            RoutablePeer { peer, address } => {
                tracing::info!(target:"network::kad", peer=%peer, addr=%address, "kad routable peer");
                DiscoveryEvent::FoundPeers(vec![(peer, address)])
            },
            PendingRoutablePeer { peer, address } => {
                tracing::info!(target:"network::kad", peer=%peer, addr=%address, "kad pending routable peer");
                DiscoveryEvent::FoundPeers(vec![(peer, address)])
            },
            UnroutablePeer { peer } => {
                tracing::info!(target:"network::kad", peer=%peer, "kad unroutable peer");
                DiscoveryEvent::PeerDown(peer)
            },
            InboundRequest { request } => {
                tracing::info!(target:"network::kad", ?request, "kad request");
                DiscoveryEvent::Ignore
            },
            OutboundQueryProgressed {id, result, stats, step} => {
                tracing::info!(target: "network::kad", %id, ?result, ?stats, ?step);
                DiscoveryEvent::Ignore
            }
            ModeChanged { new_mode } => {
                tracing::info!(target:"network::kad", %new_mode, "kad mode changed");
                DiscoveryEvent::Ignore
            }
            RoutingUpdated {peer, is_new_peer, addresses, .. } => {
                tracing::info!(target:"network::kad", %peer, %is_new_peer, ?addresses, "kad mode changed");
                DiscoveryEvent::Ignore
            }
        }

    }
}

impl From<gossipsub::Event> for DiscoveryEvent {
    fn from(event: gossipsub::Event) -> Self {
        use gossipsub::Event::*;
        match event {
            Message { message, message_id, propagation_source } => {
                tracing::info!(target:"network::gossip", message=?message, message_id=%message_id, "message comes");
                DiscoveryEvent::TxReceived
            }
            _ => { DiscoveryEvent::Ignore }
        }
    }
}

impl DiscoveryBehaviour {
    pub fn new(local_key: &identity::Keypair) -> Self {
        let public = local_key.public();
        let local_peer_id = PeerId::from(public.clone());

        let ping = ping::Behaviour::new(
            ping::Config::new().with_interval(Duration::from_secs(10))
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

        let gossipsub_cfg = gossipsub::Config::default();
        let mut gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_cfg
        ).expect("gossipsub build");

        let _ = gossipsub.subscribe(&gossipsub::IdentTopic::new("/aprova/tx"));
        let _ = gossipsub.subscribe(&gossipsub::IdentTopic::new("/aprova/block"));

        Self {
            ping,
            identify,
            mdns,
            kademlia,
            gossipsub
        }
    }

    pub fn publish_tx(&mut self) {
        let topic = gossipsub::IdentTopic::new("/aprova/tx");
        let _ = self.gossipsub.publish(topic, b"hello");
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
