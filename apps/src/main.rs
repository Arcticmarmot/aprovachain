use std::fs;
use clap::Parser;
use anyhow::{bail, Result};
use account::address::*;
use account::keypair::{AccountSigningKey, AccountVerifyingKey};
use ed25519_dalek::Signature;
use serde::{Deserialize, Serialize};
use base64::prelude::*;
use tx::tx_envelope::{TxEnvelopWire, TxEnvelope};
use tx::tx_intent::{TxIntent};
use chain::spec::{Testnet, Mainnet, ChainMetadata, ChainNetwork};
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct TxArgs {
    #[clap(short, long, env, next_help_heading = "The Chain Id of the Tx")]
    chain_id: u64,

    #[clap(short, long, env, next_help_heading = "The Account Address of the User")]
    address: String,

    #[clap(short, long, env, next_help_heading = "The Account VerifyingKey of the User")]
    verifying_key: String,

    #[clap(env, next_help_heading = "The Account SigningKey of the User")]
    signing_key: String,

    #[clap(short, long, env, next_help_heading = "The body of the Request")]
    payload_path: String,
}

fn parse_tx_args(args: TxArgs) -> Result<TxEnvelope> {
    /// build chain id from Args
    let chain_id = args.chain_id;
    println!("{}", chain_id);

    /// build vk from Args
    let mut vk_bytes = [0u8; 32];
    hex::decode_to_slice(args.verifying_key, &mut vk_bytes)?;
    let vk = AccountVerifyingKey::from_bytes(&vk_bytes)?;

    /// build addr from vk and chain_id
    let addr = match ChainNetwork::from_chain_id(chain_id) {
        ChainNetwork::Mainnet => generate_addr::<Mainnet>(&vk),
        ChainNetwork::Testnet => generate_addr::<Testnet>(&vk),
        _ => panic!("bad network id")
    };

    println!("{:?}", addr);

    /// build sk from Args
    let mut sk_bytes = [0u8; 32];
    hex::decode_to_slice(args.signing_key, &mut sk_bytes)?;
    let sk = AccountSigningKey::from_bytes(&sk_bytes);

    /// read the payload file
    let bytes = fs::read(args.payload_path)?;
    let payload = serde_json::from_slice(&bytes)?;

    let tx_intent = TxIntent::create(chain_id, addr, vk, payload)?;

    println!("\r\n{:?}", tx_intent.tx_intent_id());

    let tx_envelope = TxEnvelope::create(tx_intent, sk);

    println!("\r\n{:?}", tx_envelope.tx_id());
    Ok(tx_envelope)
}

fn generate_addr<T: ChainMetadata>(vk: &AccountVerifyingKey) -> AccountAddress<T> {
    AccountAddress::<T>::from(vk)
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

#[cfg(test)]
mod tests {
    #[test]
    fn access_args() {

    }
}
