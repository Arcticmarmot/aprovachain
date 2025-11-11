use std::error::Error;
use std::time::Duration;
use futures::StreamExt;
use libp2p::{noise, ping, tcp, mdns, yamux, Multiaddr, PeerId, identity};
use libp2p::swarm::{SwarmEvent};
use network::bootstrap::{init_env, init_logging};
use network::behaviour::discovery::{DiscoveryBehaviour, DiscoveryEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = init_logging();
    let _ = init_env();

    let key = identity::Keypair::generate_ed25519();
    let peer_id = PeerId::from(key.public());
    tracing::info!("node started: {:?}", peer_id);

    let disc_behaviour = DiscoveryBehaviour::new(&key);

    let mut swarm = libp2p::SwarmBuilder::with_existing_identity(key)
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default
        )?
        // It only cares about what messages and to whom to sent on the network.
        .with_behaviour(|_| disc_behaviour)?
        .with_swarm_config(|cfg| {
            cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX))
        })
        .build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    while let Some(event) = swarm.next().await{
        match event{
            SwarmEvent::NewListenAddr {address, ..} => {
                tracing::info!("Listening on {address:?}")
            },
            SwarmEvent::ConnectionClosed {peer_id, ..} => {
                tracing::info!("Peer down: {peer_id:?}")
            },
            SwarmEvent::Behaviour(DiscoveryEvent::PeerUp(peer_id)) => {
                tracing::info!("Peer up: {peer_id:?}")
            },
            SwarmEvent::Behaviour(DiscoveryEvent::PeerDown(peer_id)) => {
                tracing::info!("Peer down: {peer_id:?}")
            },
            SwarmEvent::Behaviour(DiscoveryEvent::FoundPeers(v)) => {
                for (peer_id, addr) in v {
                    if peer_id == *swarm.local_peer_id() { continue; } // 别拨自己
                    tracing::info!("Mdns discovered: {peer_id:?}, {addr:?}");
                    // 尝试拨过去（可能失败，忽略错误并继续）
                    if let Err(e) = swarm.dial(addr.clone()) {
                        tracing::debug!("dial {addr:?} failed: {e}");
                    }
                }
            },
            _ => {}
        }
    }

    Ok(())
}
