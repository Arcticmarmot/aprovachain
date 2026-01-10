use std::time::Duration;
use libp2p::{identify, ping, mdns, Multiaddr, PeerId, identity, kad, gossipsub, StreamProtocol};
use libp2p::gossipsub::MessageAuthenticity;
use libp2p::kad::store::MemoryStore;
use libp2p::swarm::{NetworkBehaviour};
use crate::behaviour::gossip::GossipTopic;
use crate::error::{Result};
use crate::behaviour::event::PeerEvent;
const APROVA_KAD_PROTO: &'static str = "/aprova/kad/v0.1";
const APROVA_GOSSIP_PROTO: &'static str = "/aprova/gossip/v0.1";
const HEART_BEAT_INTERVAL: u64 = 30;

pub enum PeerRole {
    Executor,
    Orderer,
    Follower,
    Verifier
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "PeerEvent")]
pub struct PeerBehaviour {
    pub ping: ping::Behaviour,
    pub identify: identify::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub gossipsub: gossipsub::Behaviour,
}

impl PeerBehaviour {
    pub fn new(local_key: &identity::Keypair, role: PeerRole) -> Self {
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
            .max_transmit_size(1024 * 1024 * 1024)
            .build().expect("build gossipsub config");
        let mut gossipsub = gossipsub::Behaviour::new(
            MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_cfg
        ).expect("gossipsub build");

        let envelope_topic = GossipTopic::Envelope.ident();
        let tx_topic = GossipTopic::Tx.ident();
        let block_topic = GossipTopic::Block.ident();
        let agreement_topic = GossipTopic::Agreement.ident();
        match role {
            PeerRole::Executor => {
                gossipsub.subscribe(&envelope_topic).expect("subscribe envelope");
                gossipsub.subscribe(&block_topic).expect("subscribe block");
            }
            PeerRole::Orderer => {
                gossipsub.subscribe(&tx_topic).expect("subscribe tx");
                gossipsub.subscribe(&block_topic).expect("subscribe block");
                gossipsub.subscribe(&agreement_topic).expect("subscribe agreement");
            }
            PeerRole::Follower => {
                gossipsub.subscribe(&agreement_topic).expect("subscribe agreement");
            }
            PeerRole::Verifier => {
                gossipsub.subscribe(&block_topic).expect("subscribe block");
            }
        }

        Self {
            ping,
            identify,
            mdns,
            kademlia,
            gossipsub
        }
    }

    pub fn publish_envelope(&mut self, envelope_bytes: Vec<u8>) -> Result<()> {
        tracing::info!(target:"network::gossip", len=%envelope_bytes.len(), "envelope size: ");
        let topic = GossipTopic::Envelope.ident();
        self.gossipsub.publish(topic, envelope_bytes)?;
        Ok(())
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
    
    pub fn publish_agreement(&mut self, agreement_bytes: Vec<u8>) -> Result<()> {
        tracing::info!(target:"network::gossip", len=%agreement_bytes.len(), "agreement size: ");
        let topic = GossipTopic::Agreement.ident();
        let _ = self.gossipsub.publish(topic, agreement_bytes);
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
