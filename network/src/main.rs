use libp2p::{identity, PeerId};

#[tokio::main]
async fn main() {
    let new_key = identity::Keypair::generate_ed25519();
    println!("New peer id: {:?}", PeerId::from(new_key.public()));
}
