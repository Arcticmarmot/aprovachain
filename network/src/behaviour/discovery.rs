use std::time::Duration;
use libp2p::{identify, ping, tcp, mdns, Multiaddr, PeerId, identity};
use libp2p::swarm::{SwarmEvent, NetworkBehaviour};

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "DiscoveryEvent")]
pub struct DiscoveryBehaviour {
    ping: ping::Behaviour,
    identify: identify::Behaviour,
    mdns: mdns::tokio::Behaviour
}

#[derive(Debug)]
pub enum DiscoveryEvent {
    PeerUp(PeerId),
    PeerDown(PeerId),
    FoundPeers(Vec<(PeerId, Multiaddr)>),
    Ignore
}

impl From<ping::Event> for DiscoveryEvent {
    fn from(event: ping::Event) -> Self {
        match &event.result {
            Ok(duration) => {
                tracing::info!("Ping success: {:?}", event);
                tracing::info!("Duration is {:?}", duration);
                DiscoveryEvent::PeerUp(event.peer)
            },
            Err(err) => {
                tracing::info!("Ping failure: {:?}", event);
                tracing::error!("Error is {:?}", err);
                DiscoveryEvent::PeerDown(event.peer)
            }
        }
    }
}

impl From<identify::Event> for DiscoveryEvent {
    fn from(event: identify::Event) -> Self {
        match event {
            identify::Event::Received {peer_id, info, ..} => {
                tracing::info!("Identify received: {:?}", peer_id);
                tracing::info!("Protocol version is: {}", info.protocol_version);
                DiscoveryEvent::PeerUp(peer_id)
            },
            identify::Event::Pushed {peer_id, ..} => {
                DiscoveryEvent::PeerUp(peer_id)
            },
            identify::Event::Sent {peer_id, ..} => {
                DiscoveryEvent::Ignore
            },
            identify::Event::Error {peer_id, error, ..} => {
                DiscoveryEvent::PeerDown(peer_id)
            },
        }
    }
}

impl From<mdns::Event> for DiscoveryEvent {
    fn from(event: mdns::Event) -> Self {
        match event {
            mdns::Event::Discovered(list) => {
                tracing::info!("{list:?}");
                DiscoveryEvent::FoundPeers(list.into_iter().collect())
            },
            mdns::Event::Expired(list) => {
                tracing::info!("{list:?}");
                DiscoveryEvent::FoundPeers(Vec::new())
            }
        }
    }
}

impl DiscoveryBehaviour {
    pub fn new(local_key: &identity::Keypair) -> Self {
        let public = local_key.public();

        // 生效的 ping 配置
        let ping = ping::Behaviour::new(
            ping::Config::new().with_interval(Duration::from_secs(5))
        );

        let identify = identify::Behaviour::new(
            identify::Config::new("/aprova/v0.1".into(), public.clone())
        );

        let mdns_cfg = mdns::Config::default();
        tracing::info!("query_interval: {:?}", mdns_cfg.query_interval);
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(), PeerId::from(public))
            .expect("mdns create failed");
        Self {
            ping,
            identify,
            mdns
        }
    }
}
