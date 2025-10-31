use std::fs;
use clap::Parser;
use anyhow::{bail, Result};
use account::address::*;
use account::keypair::{AccountSigningKey, AccountVerifyingKey};
use ed25519_dalek::Signature;
use serde::{Deserialize, Serialize};
use tx::tx_envelope::{TxEnvelope, TxEnvelopeWire};
use tx::tx_intent::{TxIntent};
use chain::spec::{ChainId, ChainSpec};
use tx::tx_envelope;
use reqwest::Client;

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

/// 从命令行参数解析出 TxEnvelopeWire
fn parse_tx_args(args: TxArgs) -> Result<TxEnvelopeWire> {
    // build chain id from Args
    let chain_id = ChainId(args.chain_id);

    // build vk from Args
    let mut vk_bytes = [0u8; 32];
    hex::decode_to_slice(args.verifying_key, &mut vk_bytes)?;
    let vk = AccountVerifyingKey::from_bytes(&vk_bytes)?;

    // build addr from chain_id and vk
    let addr = ChainAddress::create_from_vk(chain_id, &vk);

    // build sk from Args
    let mut sk_bytes = [0u8; 32];
    hex::decode_to_slice(args.signing_key, &mut sk_bytes)?;
    let sk = AccountSigningKey::from_bytes(&sk_bytes);

    // read the payload file
    // build payload from payload file
    let bytes = fs::read(args.payload_path)?;
    let payload = serde_json::from_slice(&bytes)?;

    // build tx_intend
    let tx_intent = TxIntent::create(chain_id, addr, vk, payload)?;

    // build tx_envelope
    let tx_envelope = TxEnvelope::create(tx_intent, sk);
    let tx_envelope_wire = TxEnvelopeWire::from(&tx_envelope);
    Ok(tx_envelope_wire)
}

async fn send_envelope(envelope_wire: TxEnvelopeWire) -> Result<()> {
    let client = Client::new();
    let response = client
        .post("http://localhost:8888/api/submit-tx")
        .header("content-type", "application/octet-stream")
        .body(envelope_wire.to_bcs_bytes())
        .send()
        .await?;
    println!("{:?}", response);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
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

    let tx_envelope = parse_tx_args(args)?;

    send_envelope(tx_envelope).await
}

#[cfg(test)]
mod tests {
    #[test]
    fn access_args() {

    }
}
