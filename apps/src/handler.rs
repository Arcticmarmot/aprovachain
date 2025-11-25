use std::fs;
use std::path::PathBuf;
use anyhow::{bail};
use clap::{Parser};
use reqwest::{Client, Response};
use account::address::UserAddress;
use account::keypair::{AccountSigningKey, AccountSigningKeyBytes, AccountVerifyingKey};
use primitives::file::{load_user_sk_path};
use spec::chain::ChainId;
use primitives::hash::sha256;
use tx::envelope::{TxEnvelope, TxEnvelopeWire};
use tx::intent::{TxIntent, TxPayload};

#[derive(Debug)]
pub struct TxBuildSpec {
    chain_id: ChainId,
    sk: AccountSigningKey,
    payload: TxPayload
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
pub struct TxArgs {
    #[clap(long, env, next_help_heading = "The Chain Id of the Tx")]
    chain_id: u64,

    #[clap(long, env, next_help_heading = "The payload type of TxPayload")]
    payload_type: String,

    #[clap(long, env, next_help_heading = "The deploy json of TxPayload")]
    deploy_json: Option<PathBuf>,

    #[clap(long, env, next_help_heading = "The deploy elf of TxPayload")]
    deploy_elf: Option<PathBuf>,

    #[clap(long, env, next_help_heading = "The exec json of TxPayload")]
    exec_json: Option<PathBuf>,
}

/// 从命令行参数解析出 TxEnvelopeWire
pub fn parse_tx_args(args: &TxArgs) -> anyhow::Result<TxBuildSpec> {
    parse_tx_args_with(args, generate_payload)
}

/// 自定义Payload，并从命令行参数解析出 TxEnvelopeWire
pub fn parse_tx_args_with<F>(args: &TxArgs, build_payload: F) -> anyhow::Result<TxBuildSpec>
where
    F: FnOnce(&TxArgs) -> anyhow::Result<TxPayload>
{
    // build chain id from Args
    let chain_id = ChainId(args.chain_id);

    // build sk from Args
    let sk_bytes = load_user_sk_bytes()?;
    let sk = AccountSigningKey::from_bytes(&sk_bytes);

    // build vk from sk
    let vk = sk.verifying_key();

    // build addr from vk
    let addr = UserAddress::from_vk(chain_id, &vk);

    // build payload from Args
    let payload = build_payload(args)?;

    Ok(TxBuildSpec {
        chain_id,
        sk,
        payload
    })
}

pub fn load_user_sk_bytes() -> anyhow::Result<AccountSigningKeyBytes> {
    let sk_path = load_user_sk_path();
    let sk_hex = fs::read(sk_path)?;
    let mut sk_bytes: AccountSigningKeyBytes = [0u8; 32];
    hex::decode_to_slice(sk_hex, &mut sk_bytes)?;
    Ok(sk_bytes)
}

pub fn build_envelope_wire(spec: TxBuildSpec) -> anyhow::Result<TxEnvelopeWire> {
    let tx_intent = TxIntent::create(spec.chain_id, spec.payload)?;
    let tx_envelope = TxEnvelope::create(tx_intent, spec.sk);
    let tx_envelope_wire = TxEnvelopeWire::from(&tx_envelope);
    Ok(tx_envelope_wire)
}

pub async fn send_envelope(wire: TxEnvelopeWire) -> anyhow::Result<Response> {
    let client = Client::new();
    let response = client
        .post("http://localhost:8888/api/submit-tx")
        .header("content-type", "application/octet-stream")
        .body(wire.encode_bcs())
        .send()
        .await?;
    Ok(response)
}

fn generate_payload(args: &TxArgs) -> anyhow::Result<TxPayload> {
    match args.payload_type.as_str() {
        "Deploy" => {
            generate_deploy_payload(args)
        },
        "Exec" => {
            generate_exec_payload(args)
        },
        _ => bail!("payload type mismatched")
    }
}

fn generate_deploy_payload(args: &TxArgs) -> anyhow::Result<TxPayload> {
    if let Some(elf_path) = &args.deploy_elf {
        let elf_bytes = fs::read(elf_path)?;
        let image_id = risc0_zkvm::compute_image_id(&elf_bytes)?;
        tracing::info!("{:?}", image_id);
        let elf_hash = sha256(&elf_bytes);
        tracing::info!("{:?}", elf_hash);
        let payload = TxPayload::Deploy {
            image_id,
            elf: elf_bytes,
            elf_hash
        };
        return Ok(payload)
    }
    if let Some(json) = &args.deploy_json {
        let payload_bytes = fs::read(json)?;
        let payload = serde_json::from_slice(&payload_bytes)?;
        return Ok(payload)
    }
    bail!("deploy need at least one input")
}

fn generate_exec_payload(args: &TxArgs) -> anyhow::Result<TxPayload> {
    if let Some(json) = &args.exec_json {
        let exec_bytes = fs::read(json)?;
        let payload = serde_json::from_slice(&exec_bytes)?;
        return Ok(payload)
    }
    bail!("deploy need at least one input")
}