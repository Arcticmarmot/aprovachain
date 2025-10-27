use std::fs;
use clap::Parser;
use anyhow::{bail, Result};
use account::address::*;
use account::keypair::{AccountSigningKey, AccountVerifyingKey};
use ed25519_dalek::Signature;
use serde::{Deserialize, Serialize};
use base64::prelude::*;
use tx::{Tx, TxBody, TxPayload};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct TxArgs {
    #[clap(short, long, env, next_help_heading = "The Account Address of the User")]
    address: String,

    #[clap(short, long, env, next_help_heading = "The Account VerifyingKey of the User")]
    verifying_key: String,

    #[clap(env, next_help_heading = "The Account SigningKey of the User")]
    signing_key: String,

    #[clap(short, long, env, next_help_heading = "The body of the Request")]
    payload_path: String,
}

fn parse_tx_args(args: TxArgs) -> Result<Tx> {
    // read the payload file
    let bytes = fs::read(args.payload_path)?;
    let payload = serde_json::from_slice(&bytes)?;
    println!("{:?}", payload);

    // build vk from Args
    let mut vk_bytes = [0u8; 32];
    hex::decode_to_slice(args.verifying_key, &mut vk_bytes)?;
    let vk = AccountVerifyingKey::from_bytes(&vk_bytes)?;
    let addr = AccountAddress::<UserAddress>::from(&vk);

    // build sk from Args
    let mut sk_bytes = [0u8; 32];
    hex::decode_to_slice(args.signing_key, &mut sk_bytes)?;
    let sk = AccountSigningKey::from_bytes(&sk_bytes);

    let tx_body = TxBody::new(addr, vk, payload);
    let tx = Tx::new(tx_body, sk);
    println!("{:?}", tx);
    Ok(tx)
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

    let args = TxArgs::parse();
    println!("{:?}", args);

    parse_tx_args(args)?;

    Ok(())

}
