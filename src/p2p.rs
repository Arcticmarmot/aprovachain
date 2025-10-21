use std::collections::HashSet;
use std::fmt::Display;
use tokio::net::{UdpSocket, TcpStream, TcpListener};
use std::net::{IpAddr};
use std::sync::{Arc};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use crate::common::{BROADCAST_INTERVAL_SECS, BROADCAST_IP_ADDR, LOCAL_IP_ADDR, UDP_DISC_PORT, UDP_LISTENER_PORT};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Peer {
    pub peer_id: String,
    pub ip_addr: IpAddr,
    pub listen_port: u16,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DiscPeerList {
    peers: HashSet<String>,
}

impl DiscPeerList {
    pub fn new() -> Self {
        DiscPeerList {
            peers: HashSet::default(),
        }
    }

    pub fn get_established_connection(&self) -> usize {
        self.peers.len()
    }
}

pub async fn discovery(peer: &Peer) {
    let peer_id = peer.peer_id.clone();
    let ip_addr = peer.ip_addr.clone();
    let port = peer.listen_port;

    let mut disc_peer_list = DiscPeerList::new();
    let mut shared_list = Arc::new(Mutex::new(disc_peer_list));

    let udp_disc_socket = UdpSocket::bind(format!("{}:{}", LOCAL_IP_ADDR, UDP_DISC_PORT)).await.unwrap();
    udp_disc_socket.set_broadcast(true).unwrap();
    let udp_listener_socket = UdpSocket::bind(format!("{}:{}", LOCAL_IP_ADDR, UDP_LISTENER_PORT)).await.unwrap();

    let mut listener_shared_list = shared_list.clone();
    let listener = tokio::spawn(async move {
        let mut buf = vec![0u8; 1500];
        loop {
            match udp_listener_socket.recv_from(&mut buf).await {
                Ok((n, data)) => {
                    let message = String::from_utf8_lossy(&buf[..n]).into_owned();
                    let mut shared_list = listener_shared_list.lock().await;
                    shared_list.peers.insert(message);
                },
                Err(e) => {
                    eprintln!("{:?}", e);
                    sleep(Duration::from_secs(BROADCAST_INTERVAL_SECS)).await;
                }
            }
        }
    });

    let sender_shared_list = shared_list.clone();

    let sender = tokio::spawn(async move {
        loop {
            let shared_list = sender_shared_list.lock().await;
            let established_connection = shared_list.get_established_connection();
            let broadcast_addr = format!("{}:{}", BROADCAST_IP_ADDR, UDP_LISTENER_PORT);
            let content = format!("DISCOVERY: peer_id: {}, ip_addr: {}, port: {}, established_connection: {}",
                                  peer_id, ip_addr, port, established_connection);
            println!("{} MARKS", content);
            let message = peer_id.as_bytes();
            udp_disc_socket.send_to(message, broadcast_addr).await.unwrap();
            sleep(Duration::from_secs(BROADCAST_INTERVAL_SECS)).await;
        }
    });

    let _ = tokio::join!(listener, sender);
}