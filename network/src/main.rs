use std::error::Error;
use std::num::{NonZero, NonZeroUsize};
use std::time::Duration;
use futures::StreamExt;
use libp2p::{noise, tcp, yamux, PeerId, identity, kad};
use libp2p::kad::store::MemoryStore;
use libp2p::swarm::{SwarmEvent};
use network::bootstrap::{init_env, init_logging};
use network::behaviour::discovery::{DiscoveryBehaviour, DiscoveryEvent};
use network::behaviour::peer_set::PeerSet;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = init_logging();
    let _ = init_env();

    let local_key = identity::Keypair::generate_ed25519();
    let local_id = PeerId::from(local_key.public());
    tracing::info!(target:"net::node", peer=%local_id, "node started");

    let disc_behaviour = DiscoveryBehaviour::new(&local_key);

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_tcp(tcp::Config::default(), noise::Config::new, yamux::Config::default)?
        .with_behaviour(|_| disc_behaviour)?
        .with_swarm_config(|cfg| { cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)) })
        .build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/33333".parse()?)?;

    // 创建 PeerSet 记录在线节点
    let mut peer_set = PeerSet::new(local_id);
    let _ = PeerSet::init(&mut swarm);

    while let Some(event) = swarm.next().await{
        match event{
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
            SwarmEvent::Behaviour(DiscoveryEvent::PeerUp(peer_id, addrs_opt)) => {
                tracing::info!(target:"net::disc", peer=%peer_id, addrs=?addrs_opt, "peer up");
                swarm.behaviour_mut().kad_peer_up(&peer_id, addrs_opt.clone());
                peer_set.on_peer_up(peer_id, addrs_opt);
                peer_set.refresh(&mut swarm);
            },
            SwarmEvent::Behaviour(DiscoveryEvent::PeerDown(peer_id)) => {
                tracing::info!(target:"net::disc", peer=%peer_id, "peer down");
                peer_set.on_peer_down(peer_id);
                peer_set.refresh(&mut swarm);
            },
            SwarmEvent::Behaviour(DiscoveryEvent::FoundPeers(peers)) => {
                tracing::info!(target:"net::disc", known_peers=peers.len(), "mdns discovered candidates");
                swarm.behaviour_mut().kad_found_peers(&peers);
                peer_set.on_found_peers(peers);
                peer_set.refresh(&mut swarm);
            },
            _ => {}
        }
    }
    Ok(())
}
