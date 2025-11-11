use std::collections::HashMap;
use libp2p::PeerId;

#[derive(Debug)]
pub struct PeerSet {
    peers: HashMap<PeerId, PeerInfo>
}

#[derive(Debug)]
pub struct PeerInfo {
    
}