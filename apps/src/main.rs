use std::fs;
use clap::Parser;
use anyhow::{bail, Result};
use account::address::*;
use account::keypair::AccountVerifyingKey;
use ed25519_dalek::Signature;
use rpassword::prompt_password;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteTx {
    nonce: u64,
    image_id: String,
    input: String
}

pub struct TxEnvelope {

}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Args {
    #[clap(short, long, env, next_help_heading = "The Account Address of the User")]
    address: String,

    #[clap(short, long, env, next_help_heading = "The Account VerifyingKey of the User")]
    verifying_key: String,

    #[clap(env, next_help_heading = "The Account SigningKey of the User")]
    signing_key: String,

    #[clap(short, long, env, next_help_heading = "The body of the Request")]
    execute_tx_path: String,
}



fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    match dotenvy::dotenv() {
        Ok(path) => tracing::debug!("Loaded environment variables from {:?}", path),
        Err(e) if e.not_found() => tracing::error!("No .env found"),
        Err(e) => bail!("failed to load .env file: {}", e),
    }

    let args = Args::parse();

    println!("{:?}", args);

    let bytes = fs::read(args.execute_tx_path)?;

    let execute_tx: ExecuteTx = serde_json::from_slice(&bytes)?;

    println!("{:?}", execute_tx);

    let vk_bytes = hex::decode(args.verifying_key)?;
    let mut vk_source = [0u8; 32];
    vk_source.copy_from_slice(&vk_bytes);
    let vk = AccountVerifyingKey::from_bytes(&vk_source)?;
    let addr = AccountAddress::<UserAddress>::from(&vk);
    println!("{:?}", vk);
    println!("{}", addr.to_string());
    println!("{}", args.address);
    Ok(())

}
