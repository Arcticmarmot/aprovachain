use std::time::Duration;
use libp2p::{identify, ping, mdns, Multiaddr, PeerId, identity};
use libp2p::swarm::{NetworkBehaviour};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "DiscoveryEvent")]
pub struct DiscoveryBehaviour {
    ping: ping::Behaviour,
    identify: identify::Behaviour,
    mdns: mdns::tokio::Behaviour,
    // kademlia: kad::Behaviour<kad::store::MemoryStore>
}

#[derive(Debug)]
pub enum DiscoveryEvent {
    PeerUp(PeerId, Option<Vec<Multiaddr>>),
    PeerDown(PeerId),
    FoundPeers(Vec<(PeerId, Multiaddr)>),
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
                tracing::debug!(target:"network::mdns", count=peers.len(), "mdns expired peers");
                tracing::trace!(target:"network::mdns", ?peers, "mdns expired details");
                DiscoveryEvent::Ignore
            }
        }
    }
}

impl DiscoveryBehaviour {
    pub fn new(local_key: &identity::Keypair) -> Self {
        let public = local_key.public();

        let ping = ping::Behaviour::new(
            ping::Config::new().with_interval(Duration::from_secs(20))
        );

        let identify = identify::Behaviour::new(
            identify::Config::new("/aprova/v0.1".into(), public.clone())
        );

        let mdns_cfg = mdns::Config::default();
        let mdns = mdns::tokio::Behaviour::new(
            mdns_cfg, PeerId::from(public))
            .expect("mdns create failed");
        Self {
            ping,
            identify,
            mdns
        }
    }
}
