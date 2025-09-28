use std::fmt::format;
use tokio::net::{UdpSocket, TcpStream, TcpListener};
use std::net::Ipv4Addr;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};


#[derive(Serialize, Deserialize, Debug, Clone)]
struct Peer {
    addr: Ipv4Addr,
    port: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct DiscoveryMsg {
    addr: Ipv4Addr,
    port: usize,
}

const LOCAL_IP_ADDR: &'static str = "0.0.0.0";
const BROADCAST_IP_ADDR: &'static str = "192.168.1.255";
const UDP_DISC_PORT: usize = 9000;
const UDP_LISTENER_PORT: usize = 9001;

const BROADCAST_INTERVAL_SECS: u64 = 5;
pub async fn discovery(id: String) {
    let udp_disc_socket = UdpSocket::bind(format!("{}:{}", LOCAL_IP_ADDR, UDP_DISC_PORT)).await.unwrap();
    udp_disc_socket.set_broadcast(true).unwrap();
    let udp_listener_socket = UdpSocket::bind(format!("{}:{}", LOCAL_IP_ADDR, UDP_LISTENER_PORT)).await.unwrap();


    let listener = tokio::spawn(async move {
        let mut buf = vec![0u8; 1500];
        loop {
            match udp_listener_socket.recv_from(&mut buf).await {
                Ok((n, data)) => {
                    let message = String::from_utf8_lossy(&buf[..n]);
                    println!("{:?}", message);
                },
                Err(e) => {
                    eprintln!("{:?}", e);
                    sleep(Duration::from_secs(BROADCAST_INTERVAL_SECS)).await;
                }
            }
        }
    });

    let sender = tokio::spawn(async move {
        loop {
            let content = "DISCOVERY: ".to_owned() + &id;
            let message = content.as_bytes();
            let broadcast_addr = format!("{}:{}", BROADCAST_IP_ADDR, UDP_LISTENER_PORT);
            udp_disc_socket.send_to(message, broadcast_addr).await.unwrap();
            sleep(Duration::from_secs(BROADCAST_INTERVAL_SECS)).await;
        }
    });

    let _ = tokio::join!(listener, sender);
}