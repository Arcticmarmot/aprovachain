use std::cmp::{min};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use libp2p::{Multiaddr, PeerId, Swarm};
use libp2p::swarm::dial_opts::{DialOpts, PeerCondition};
use libp2p::swarm::{DialError, NetworkBehaviour};
use crate::error::{PeerError, Result};
use rand::rng;
use rand::seq::IteratorRandom;

const BACKOFF: u64 = 5;
const PURGE: u64 = 20;
/// 每次最大 dial 地址数量
const MAX_ADDR_DIAL: usize = 3;
const BOOTNODE_ADDRS: &[&'static str] = &[
    "/ip4/100.64.250.18/tcp/33333",
    "/ip4/100.107.181.54/tcp/33333",
];
const BOOTNODE_IDS: &[PeerId] = &[ ];

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


pub fn dial<B>(peer_id: &PeerId, addrs: &HashSet<Multiaddr>, swarm: &mut Swarm<B>) -> Result<()>
where B: NetworkBehaviour
{
    let peer_dial_opts = DialOpts::peer_id(*peer_id).condition(PeerCondition::Disconnected).build();
    match swarm.dial(peer_dial_opts) {
        Ok(_) => {
            tracing::info!(target: "network::dial", %peer_id, "dial by peer id ok");
            return Ok(())
        },
        Err(err)if matches!(err, DialError::DialPeerConditionFalse(_))  => {
            tracing::error!(target: "network::dial", %peer_id, ?err);
            return Err(PeerError::DialPeer)
        },
        Err(err)if matches!(err, DialError::NoAddresses)  => {
            tracing::error!(target: "network::dial", %peer_id, ?err);
            return Err(PeerError::DialPeer)
        },
        Err(err) => {
            tracing::error!(target: "network::dial", %peer_id, ?err);
        }
    }
    // 随机选取 MAX_ADDR_DIAL 个地址
    let mut rng = rng();
    tracing::info!(target: "network::dial", addrs_len=%addrs.len(), "dial addrs len");
    let dial_addrs = addrs.iter().choose_multiple(&mut rng, min(MAX_ADDR_DIAL, addrs.len()));
    let mut flag = false;
    for addr in dial_addrs {
        let addr_dial_opts = DialOpts::unknown_peer_id().address(addr.clone()).build();
        match swarm.dial(addr.clone()) {
            Ok(_) => {
                tracing::info!(target: "network::dial", %peer_id, %addr, "dial by addr ok");
                flag = true
            }
            Err(err) => {
                tracing::error!(target: "network::dial", %peer_id, %addr, ?err, "dial by addr failed");
            }
        }
    }
    if flag { Ok(()) } else { Err(PeerError::DialPeer) }
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

    pub fn init<B>(&mut self, swarm: &mut Swarm<B>) -> Result<()> where B: NetworkBehaviour {
        for str_addr in BOOTNODE_ADDRS {
            self.add_bootnode(None, str_addr);
        }
        self.refresh(swarm);
        Ok(())
    }

    pub fn add_bootnode(&mut self, peer_id: Option<PeerId>, addr_str: &'static str) {
        let peer_id = peer_id.unwrap_or(PeerId::random());
        let mut addrs = HashSet::new();
        addrs.insert(addr_str.parse::<Multiaddr>().expect("invalid address"));
        self.map.insert(peer_id, PeerInfo {
            addrs,
            last_dial: None,
            last_seen: None,
        });
    }

    pub fn refresh<B>(&mut self, swarm: &mut Swarm<B>) where B: NetworkBehaviour {
        let now = Instant::now();
        // 清理没动静的节点
        self.map.retain(|id, info| {
            if id == &self.local_id { return false }
            match (info.last_seen, info.last_dial) {
                (Some(seen), _) => Self::is_in_purge(now, seen, self.purge),
                (None, Some(dial)) => Self::is_in_purge(now, dial, self.purge),
                (None, None) => true
            }
        });
        // 对所有节点
        for (id, info) in self.map.iter_mut() {
            if id == &self.local_id { continue; }
            if info.last_dial.is_none() {
                tracing::info!("OK LET'S START");
                match dial::<B>(id, &info.addrs, swarm) {
                    Ok(()) => { info.last_dial = Some(Instant::now()) }
                    _ => { }
                }
                continue;
            }
            if let Some(dial) = info.last_dial {
                if Self::is_in_backoff(now, dial, self.backoff) { continue; }
            }
            if let Some(seen) = info.last_seen {
                if Self::is_in_backoff(now, seen, self.backoff) { continue; }
            }
            tracing::info!("OK LET'S START");
            match dial::<B>(id, &info.addrs, swarm) {
                Ok(()) => { info.last_dial = Some(Instant::now()) }
                _ => { }
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

    pub fn is_in_backoff(now: Instant, moment: Instant, backoff: Duration) -> bool {
        now.saturating_duration_since(moment) < backoff
    }

    pub fn is_in_purge(now: Instant, moment: Instant, purge: Duration) -> bool {
        now.saturating_duration_since(moment) < purge
    }
}