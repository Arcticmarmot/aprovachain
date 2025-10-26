use std::path::{PathBuf};
use account::address::{AccountAddress, UserAddress};
use account::keypair::*;

fn main() {
    let keypair = Keypair::generate();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let keypair_dir = root.join("keypair");
    keypair.save_address::<UserAddress>(&keypair_dir, "address.txt").expect("saving the address to file");
    keypair.save_sk_hex(&keypair_dir, "signing-key").expect("saving the signing key to file");
    keypair.save_vk_hex(&keypair_dir, "verifying-key").expect("saving the verifying key to file");
}