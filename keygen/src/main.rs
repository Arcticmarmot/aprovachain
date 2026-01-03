use std::path::{PathBuf};
use libp2p::PeerId;
use account::keypair::*;
use platform::file::aprova_proj_dir;

fn main() {
    let proj = aprova_proj_dir().expect("proj dir not found");
    let keypair_path = proj.data_local_dir().join("keypair");
    gen_user_keypair(&keypair_path);
    gen_node_keypair(&keypair_path);
}

fn gen_user_keypair(keypair_path: &PathBuf) {
    let keypair = Keypair::generate();
    let user_dir = keypair_path.join("user");
    keypair.save_address(&user_dir, "address.txt", 1000).expect("saving the address to file");
    keypair.save_sk_hex(&user_dir, "signing-key").expect("saving the signing key to file");
    keypair.save_vk_hex(&user_dir, "verifying-key").expect("saving the verifying key to file");
}

fn gen_node_keypair(keypair_path: &PathBuf) {
    let keypair = Keypair::generate();
    let keypair_dir = keypair_path.join("node");
    keypair.save_sk_hex(&keypair_dir, "signing-key").expect("saving the signing key to file");
    keypair.save_vk_hex(&keypair_dir, "verifying-key").expect("saving the verifying key to file");
    let peer_key = libp2p::identity::Keypair::ed25519_from_bytes(keypair.signing_key.to_bytes()).unwrap();
    let peer_id_str = PeerId::from(peer_key.public()).to_base58();
    keypair.save_peer_id(&keypair_dir, "peer-id", peer_id_str).expect("saving the peer id to file");
}