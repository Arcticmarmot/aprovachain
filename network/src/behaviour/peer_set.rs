use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use libp2p::{Multiaddr, PeerId, Swarm};
use libp2p::swarm::dial_opts::{DialOpts, PeerCondition};
use libp2p::swarm::{NetworkBehaviour};
use crate::error::{PeerError, Result};

const BACKOFF: u64 = 5;
const PURGE: u64 = 20;
const BOOTNODE_ADDRS: &[&'static str] = &[
    "/ip4/100.64.250.18/tcp/33333", // belgrade
    "/ip4/100.107.181.54/tcp/33333", // mecca
    "/ip4/100.70.254.75/tcp/33333", // minsk
    "/ip4/100.105.138.1/tcp/33333", // kol-server
    "/ip4/100.94.178.96/tcp/33333", // smolensk
];

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

    pub fn clear(&mut self) {
        let now = Instant::now();
        // 清理没动静的节点
        self.map.retain(|id, info| {
            if id == &self.local_id { return false }
            match (info.last_seen, info.last_dial) {
                (Some(seen), _) => Self::in_duration(now, seen, self.purge),
                (None, Some(dial)) => Self::in_duration(now, dial, self.purge),
                (None, None) => true
            }
        });
    }

    pub fn refresh<B>(&mut self, swarm: &mut Swarm<B>) where B: NetworkBehaviour {
        self.clear();
        let now = Instant::now();
        for (id, info) in self.map.iter_mut() {
            if id == &self.local_id { continue; }
            if let Some(seen) = info.last_seen {
                if Self::in_duration(now, seen, self.backoff) { continue; }
            }
            match info.last_dial {
                Some(dial) => {
                    if Self::in_duration(now, dial, self.backoff) { continue }
                }
                None => { }
            }
            tracing::debug!(target: "network::dial", "start dial");
            match Self::dial(id, &info.addrs, swarm) {
                Ok(()) => { info.last_dial = Some(Instant::now()) }
                _ => { }
            }
        }
        tracing::info!(target: "network::peer-set", peer_count=?self.map.len());
    }

    pub fn dial<B>(peer_id: &PeerId, addrs: &HashSet<Multiaddr>, swarm: &mut Swarm<B>) -> Result<()> where B: NetworkBehaviour {
        let addrs = addrs.iter().map(|addr| addr.clone()).collect::<Vec<Multiaddr>>();
        let peer_dial_opts = DialOpts::peer_id(*peer_id).addresses(addrs).condition(PeerCondition::Disconnected).build();
        match swarm.dial(peer_dial_opts) {
            Ok(_) => {
                tracing::info!(target: "network::dial", %peer_id, "dial by peer id ok");
                Ok(())
            },
            Err(err) => {
                tracing::warn!(target: "network::dial", %peer_id, ?err);
                Err(PeerError::DialPeer)
            }
        }
    }

    pub fn init<B>(swarm: &mut Swarm<B>) -> Result<()> where B: NetworkBehaviour {
        for addr_str in BOOTNODE_ADDRS {
            let addr = addr_str.parse::<Multiaddr>().expect("invalid address");
            let addr_dial_opts = DialOpts::unknown_peer_id().address(addr.clone()).build();
            match swarm.dial(addr_dial_opts) {
                Ok(_) => {
                    tracing::info!(target: "network::dial", %addr, "dial by bootnode addr ok");
                }
                Err(err) => {
                    tracing::error!(target: "network::dial", %addr, ?err, "dial by bootnode addr failed");
                }
            }
        }
        Ok(())
    }

    pub fn in_duration(now: Instant, moment: Instant, duration: Duration) -> bool {
        now.saturating_duration_since(moment) < duration
    }

    /// 排除 IPv4 本机回环地址 127.0.0.1 和 Docker专用地址 172.17.*.*
    pub fn insert_addr(addrs: &mut HashSet<Multiaddr>, addr: Multiaddr) {
        let addr_vec = addr.to_vec();
        if addr_vec.len() >= 5 && addr_vec[0] == 4 && addr_vec[1] == 127 { return; }
        if addr_vec.len() >= 5 && addr_vec[0] == 4 && addr_vec[1] == 172 && addr_vec[2] == 17 { return; }
        addrs.insert(addr);
    }

    pub fn on_peer_up(&mut self, peer_id: PeerId, addrs_opt: Option<Vec<Multiaddr>>) {
        let ele = self.map.entry(peer_id).or_default();
        ele.last_seen = Some(Instant::now());
        if let Some(addrs) = addrs_opt {
            for addr in addrs {
                Self::insert_addr(&mut ele.addrs, addr);
            }
        }
    }

    pub fn on_found_peers(&mut self, peers: Vec<(PeerId, Multiaddr)>) {
        for (peer_id, addr) in peers {
            let ele = self.map.entry(peer_id).or_default();
            ele.last_seen = Some(Instant::now());
            Self::insert_addr(&mut ele.addrs, addr);
        }
    }
}