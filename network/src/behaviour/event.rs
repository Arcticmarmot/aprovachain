use libp2p::{gossipsub, identify, kad, mdns, ping, Multiaddr, PeerId};
use libp2p::gossipsub::{Message, MessageId};

#[derive(Debug)]
pub enum PeerEvent {
    PeerUp(PeerId, Option<Vec<Multiaddr>>),
    PeerDown(PeerId),
    FoundPeers(Vec<(PeerId, Multiaddr)>),
    MessageReceived(PeerId, MessageId, Message),
    Ignore
}

impl From<ping::Event> for PeerEvent {
    fn from(event: ping::Event) -> Self {
        match &event.result {
            Ok(rtt) => {
                tracing::debug!(target:"net::ping", peer=%event.peer, ?rtt, "ping ok");
                PeerEvent::PeerUp(event.peer, None)
            },
            Err(err) => {
                tracing::info!(target:"net::ping", peer=%event.peer, ?err, "ping failed");
                PeerEvent::PeerDown(event.peer)
            }
        }
    }
}

impl From<identify::Event> for PeerEvent {
    fn from(event: identify::Event) -> Self {
        match event {
            identify::Event::Received {peer_id, info, ..} => {
                tracing::info!(target:"net::identify", %peer_id, "identify received");
                tracing::debug!(target:"net::identify",
                    %peer_id, agent=%info.agent_version, listen_addrs=?info.listen_addrs, protocols=?info.protocols, observed_addr=%info.observed_addr,
                    "identify details");
                let addrs = info.listen_addrs;
                PeerEvent::PeerUp(peer_id, Some(addrs)) // 传递对端自报的地址
            },
            identify::Event::Pushed {peer_id, info, ..} => {
                tracing::info!(target:"net::identify", %peer_id, "identify received");
                tracing::debug!(target:"net::identify",
                    %peer_id, agent=%info.agent_version, listen_addrs=?info.listen_addrs, protocols=?info.protocols, observed_addr=%info.observed_addr,
                    "identify details");
                let addrs = info.listen_addrs;
                PeerEvent::PeerUp(peer_id, Some(addrs)) // 传递对端自报的地址
            },
            identify::Event::Sent { .. } => {
                PeerEvent::Ignore
            },
            identify::Event::Error {peer_id, error, ..} => {
                tracing::warn!(target:"net::identify", %peer_id, ?error, "identify error");
                PeerEvent::PeerDown(peer_id)
            },
        }
    }
}

impl From<mdns::Event> for PeerEvent {
    fn from(event: mdns::Event) -> Self {
        match event {
            mdns::Event::Discovered(peers) => {
                tracing::info!(target:"network::mdns", count=peers.len(), "mdns discovered peers");
                tracing::debug!(target:"network::mdns", ?peers, "mdns discovered details");
                PeerEvent::FoundPeers(peers.into_iter().collect())
            },
            mdns::Event::Expired(peers) => {
                tracing::info!(target:"network::mdns", count=peers.len(), "mdns expired peers");
                tracing::trace!(target:"network::mdns", ?peers, "mdns expired details");
                PeerEvent::Ignore
            }
        }
    }
}

impl From<kad::Event> for PeerEvent {
    fn from(event: kad::Event) -> Self {
        use kad::Event::*;
        match event {
            RoutablePeer { peer, address } => {
                tracing::info!(target:"network::kad", peer=%peer, addr=%address, "kad routable peer");
                PeerEvent::FoundPeers(vec![(peer, address)])
            },
            PendingRoutablePeer { peer, address } => {
                tracing::info!(target:"network::kad", peer=%peer, addr=%address, "kad pending routable peer");
                PeerEvent::FoundPeers(vec![(peer, address)])
            },
            UnroutablePeer { peer } => {
                tracing::info!(target:"network::kad", peer=%peer, "kad unroutable peer");
                PeerEvent::PeerDown(peer)
            },
            InboundRequest { request } => {
                tracing::debug!(target:"network::kad", ?request, "kad request");
                PeerEvent::Ignore
            },
            OutboundQueryProgressed {id, result, stats, step} => {
                tracing::debug!(target: "network::kad", %id, ?result, ?stats, ?step);
                PeerEvent::Ignore
            }
            ModeChanged { new_mode } => {
                tracing::debug!(target:"network::kad", %new_mode, "kad mode changed");
                PeerEvent::Ignore
            }
            RoutingUpdated {peer, is_new_peer, addresses, .. } => {
                tracing::debug!(target:"network::kad", %peer, %is_new_peer, ?addresses, "kad routing update");
                PeerEvent::Ignore
            }
        }

    }
}

impl From<gossipsub::Event> for PeerEvent {
    fn from(event: gossipsub::Event) -> Self {
        use gossipsub::Event::*;
        match event {
            Message { message, message_id, propagation_source } => {
                tracing::info!(target:"network::gossip", source=%propagation_source, message=?message, message_id=%message_id, "message comes");
                PeerEvent::MessageReceived(propagation_source, message_id, message)
            },
            Subscribed {peer_id, topic} => {
                tracing::info!(target:"network::gossip", %peer_id, %topic, "subscribe");
                PeerEvent::Ignore
            },
            Unsubscribed {peer_id, topic} => {
                tracing::info!(target:"network::gossip", %peer_id, %topic, "unsubscibe");
                PeerEvent::Ignore
            },
            GossipsubNotSupported {peer_id} => {
                tracing::info!(target:"network::gossip", %peer_id, "unsupported");
                PeerEvent::Ignore
            },
            SlowPeer {peer_id, failed_messages} => {
                tracing::info!(target:"network::gossip", %peer_id, ?failed_messages, "unsupported");
                PeerEvent::Ignore
            }
        }
    }
}