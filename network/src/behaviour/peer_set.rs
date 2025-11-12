use std::collections::{HashMap, HashSet};
use std::time::Instant;
use libp2p::{Multiaddr, PeerId, Swarm};

#[derive(Debug)]
pub struct PeerSet {
    pub map: HashMap<PeerId, PeerInfo>
}

#[derive(Default, Debug)]
pub struct PeerInfo {
    pub addrs: HashSet<Multiaddr>,
    pub is_online: bool,
    pub last_seen: Option<Instant>
}

impl PeerSet {
    pub fn new() -> Self {
        Self {
            map: HashMap::new()
        }
    }

    pub fn refresh<B>(&self, swarm: Swarm<B>)
        where B: libp2p::swarm::NetworkBehaviour
    {
        let map = &self.map;
        for (id, info) in map {
            if info.is_online { continue; }
            let now = Instant::now();
            if now.duration_since(info.last_seen.unwrap()) >= 20 {

            }
        }
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
            ele.is_online = true;
            ele.last_seen = Some(Instant::now());
            ele.addrs.insert(addr);
        }
    }
}