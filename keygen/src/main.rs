use std::path::{PathBuf};
use account::keypair::*;

fn main() {
    let keypair = Keypair::generate();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let keypair_dir = root.join("keypair");
    keypair.save_address(&keypair_dir, "address.txt", 1000).expect("saving the address to file");
    keypair.save_sk_hex(&keypair_dir, "signing-key").expect("saving the signing key to file");
    keypair.save_vk_hex(&keypair_dir, "verifying-key").expect("saving the verifying key to file");
}