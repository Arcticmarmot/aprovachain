use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use libp2p::{Multiaddr, PeerId, Swarm};

const BACKOFF: u64 = 5;
const PURGE: u64 = 20;

#[derive(Debug)]
pub struct PeerSet {
    pub map: HashMap<PeerId, PeerInfo>,
    pub local_id: PeerId,
    pub backoff: Duration,
    pub purge: Duration
}

#[derive(Default, Debug)]
pub struct PeerInfo {
    pub addrs: HashSet<Multiaddr>,
    pub last_dial: Option<Instant>,
    pub last_seen: Option<Instant>,
}

impl PeerSet {
    pub fn new(local_id: PeerId) -> Self {
        Self {
            map: HashMap::new(),
            local_id,
            backoff: Duration::from_secs(BACKOFF),
            purge: Duration::from_secs(PURGE)
        }
    }

    pub fn with_config(local_id: PeerId, backoff: Duration, purge: Duration) -> Self {
        Self {
            map: HashMap::new(),
            local_id,
            backoff,
            purge
        }
    }

    pub fn add_bootnodes(&mut self, peer_id: PeerId, addr: Multiaddr) {
        self.map.insert(peer_id, PeerInfo {
            addrs: HashSet::from([addr]),
            last_dial: None,
            last_seen: None
        });
    }

    pub fn refresh<B>(&mut self, swarm: &mut Swarm<B>)
        where B: libp2p::swarm::NetworkBehaviour
    {
        let now = Instant::now();

        // 清理没动静的节点
        self.map.retain(|id, info| {
            if id == &self.local_id { return false }
            match (info.last_seen, info.last_dial) {
                (Some(seen), _) => now.saturating_duration_since(seen) < self.purge,
                (None, Some(dial)) => now.saturating_duration_since(dial) < self.purge,
                (None, None) => true
            }
        });

        for (id, info) in self.map.iter_mut() {
            if id == &self.local_id { continue; }

            match info.last_dial {
                Some(dial) => {
                    let pass = now.saturating_duration_since(dial);
                    if pass < self.backoff { continue; }
                    if let Some(seen) = info.last_seen {
                        let pass = now.saturating_duration_since(seen);
                        if pass < self.backoff { continue; }
                    }
                }
                None => {
                    tracing::info!("OK LET'S START REFRESH");
                    for addr in &info.addrs {
                        tracing::info!(target: "net::dial", %addr, "dial addr");
                        match swarm.dial(addr.clone()) {
                            Ok(()) => {
                                info.last_dial = Some(Instant::now());
                            },
                            Err(err) => {
                                tracing::warn!(target: "net::dial", ?err, "dial failed")
                            }
                        };
                    }
                }
            }
        }
        tracing::info!(target: "network::peer-set", len=?self.map.len());
    }

    pub fn on_peer_up(&mut self, peer_id: PeerId, addrs_opt: Option<Vec<Multiaddr>>) {
        let ele = self.map.entry(peer_id).or_default();
        ele.last_seen = Some(Instant::now());
        if let Some(addrs) = addrs_opt {
            for addr in addrs {
                ele.addrs.insert(addr);
            }
        }
    }

    pub fn on_peer_down(&mut self, peer_id: PeerId) {
        // let ele = self.map.entry(peer_id).or_default();
    }

    pub fn on_found_peers(&mut self, peers: Vec<(PeerId, Multiaddr)>) {
        for (peer_id, addr) in peers {
            let ele = self.map.entry(peer_id).or_default();
            ele.last_seen = Some(Instant::now());
            ele.addrs.insert(addr);
        }
    }
}