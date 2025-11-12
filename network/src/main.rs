use std::error::Error;
use std::time::Duration;
use futures::StreamExt;
use libp2p::{noise, tcp, yamux, Multiaddr, PeerId, identity};
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

    let mut peer_set = PeerSet::new(local_id);
    peer_set.add_bootnode("12D3KooWAuNqc1Y5HkLzXt3n2RoM4ryHoYJeMwg1CmRdWQGzLzYC".parse()?,
                          "/ip4/100.107.181.54/tcp/33333".parse()?);
    peer_set.refresh(&mut swarm);
    while let Some(event) = swarm.next().await{
        match event{
            SwarmEvent::NewListenAddr {address, ..} => {
                tracing::info!(target:"net::listen", addr=%address, "listening");
            }
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                tracing::info!(target:"net::conn", peer=%peer_id, "connection established");
            }
            SwarmEvent::ConnectionClosed {peer_id, ..} => {
                tracing::info!(target:"net::conn", peer=%peer_id, "connection closed");
            }
            // DiscoveryEvent 事件处理
            SwarmEvent::Behaviour(DiscoveryEvent::PeerUp(peer_id, addrs_opt)) => {
                tracing::info!(target:"net::disc", peer=%peer_id, addrs=?addrs_opt, "peer up");
                peer_set.on_peer_up(peer_id, addrs_opt);
                peer_set.refresh(&mut swarm);
            },
            SwarmEvent::Behaviour(DiscoveryEvent::PeerDown(peer_id)) => {
                tracing::warn!(target:"net::disc", peer=%peer_id, "peer down");
                peer_set.on_peer_down(peer_id);
                peer_set.refresh(&mut swarm);
            },
            SwarmEvent::Behaviour(DiscoveryEvent::FoundPeers(peers)) => {
                tracing::debug!(target:"net::disc", known_peers=peers.len(), "mdns discovered candidates");
                peer_set.on_found_peers(peers);
                peer_set.refresh(&mut swarm);
            },
            _ => {}
        }
    }
    Ok(())
}
