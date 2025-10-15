use crate::common::{calc_peer_hash, get_default_outbound_ip, UDP_DISC_PORT};
use crate::p2p::{discovery, DiscPeerList, Peer};

mod p2p;
mod app;
mod common;
#[tokio::main]
async fn main() {
    let local_ip_addr = get_default_outbound_ip().unwrap();
    let local_default_port = UDP_DISC_PORT;
    let local_peer_id = calc_peer_hash(local_ip_addr, local_default_port);
    let local_peer = Peer {
        peer_id: local_peer_id,
        ip_addr: local_ip_addr,
        listen_port: local_default_port,
    };
    println!("{:?}", local_peer);
    discovery(&local_peer).await;
}
