use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use libp2p::{Multiaddr, PeerId, Swarm};

const BACKOFF: u64 = 10;
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
    pub is_online: bool,
    pub last_seen: Option<Instant>
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

    pub fn add_bootnode(&mut self, peer_id: PeerId, addr: Multiaddr) {
        self.map.insert(peer_id, PeerInfo {
            addrs: HashSet::from([addr]),
            is_online: false,
            last_seen: None
        });
    }

    pub fn refresh<B>(&mut self, swarm: &mut Swarm<B>)
        where B: libp2p::swarm::NetworkBehaviour
    {
        let now = Instant::now();
        self.map.retain(|id, info| {
            if id == &self.local_id { return false }
            match info.last_seen {
                Some(seen) => now.saturating_duration_since(seen) < self.purge,
                None => true
            }
        });
        let map = &mut self.map;
        for (id, info) in map {
            if id == &self.local_id { continue; }
            if info.is_online { continue; }
            match info.last_seen {
                Some(seen) => {
                    let pass = now.saturating_duration_since(seen);
                    tracing::info!(target: "network::set", ?pass);
                    if pass <= self.backoff || pass >= self.purge {
                        continue;
                    }
                },
                None => {
                    info.last_seen = Some(Instant::now())
                }
            }
            for addr in info.addrs.iter() {
                let _ =swarm.dial(addr.clone());
            }
        }
        tracing::info!(target: "network::peer-set", map=?self.map);
    }

    pub fn on_peer_up(&mut self, peer_id: PeerId, addrs_opt: Option<Vec<Multiaddr>>) {
        let ele = self.map.entry(peer_id).or_default();
        ele.is_online = true;
        ele.last_seen = Some(Instant::now());
        if let Some(addrs) = addrs_opt {
            for addr in addrs {
                ele.addrs.insert(addr);
            }
        }
    }

    pub fn on_peer_down(&mut self, peer_id: PeerId) {
        let ele = self.map.entry(peer_id).or_default();
        ele.is_online = false;
    }

    pub fn on_found_peers(&mut self, peers: Vec<(PeerId, Multiaddr)>) {
        for (peer_id, addr) in peers {
            let ele = self.map.entry(peer_id).or_default();
            ele.last_seen = Some(Instant::now());
            ele.addrs.insert(addr);
        }
    }
}