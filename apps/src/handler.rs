use std::fs;
use std::path::PathBuf;
use anyhow::{bail};
use clap::{Parser};
use reqwest::Client;
use account::address::ChainAddress;
use account::keypair::{AccountSigningKey, AccountVerifyingKey};
use chain::spec::ChainId;
use primitives::hash::sha256;
use tx::tx_envelope::{TxEnvelope, TxEnvelopeWire};
use tx::tx_intent::{TxIntent, TxPayload};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
pub struct TxArgs {
    #[clap(long, env, next_help_heading = "The Chain Id of the Tx")]
    chain_id: u64,

    #[clap(long, env, next_help_heading = "The Account Address of the User")]
    address: String,

    #[clap(long, env, next_help_heading = "The Account VerifyingKey of the User")]
    verifying_key: String,

    #[clap(long, env, next_help_heading = "The Account SigningKey of the User")]
    signing_key: String,

    #[clap(long, env, next_help_heading = "The body of the Request")]
    payload_type: String,

    #[clap(long, env, next_help_heading = "The body of the Request")]
    deploy_json: Option<PathBuf>,

    #[clap(long, env, next_help_heading = "The body of the Request")]
    deploy_elf: Option<PathBuf>,

    #[clap(long, env, next_help_heading = "The body of the Request")]
    exec_json: Option<PathBuf>,
}

/// 从命令行参数解析出 TxEnvelopeWire
pub fn parse_tx_args(args: &TxArgs) -> anyhow::Result<TxEnvelopeWire> {
    parse_tx_args_with(args, generate_payload)
}

/// 自定义Payload，并从命令行参数解析出 TxEnvelopeWire
pub fn parse_tx_args_with<F>(args: &TxArgs, build_payload: F) -> anyhow::Result<TxEnvelopeWire>
where
    F: FnOnce(&TxArgs) -> anyhow::Result<TxPayload>
{
    // build chain id from Args
    let chain_id = ChainId(args.chain_id);

    // build vk from Args
    let mut vk_bytes = [0u8; 32];
    hex::decode_to_slice(&args.verifying_key, &mut vk_bytes)?;
    let vk = AccountVerifyingKey::from_bytes(&vk_bytes)?;

    // build addr from chain_id and vk
    let addr = ChainAddress::create_from_vk(chain_id, &vk);

    // build sk from Args
    let mut sk_bytes = [0u8; 32];
    hex::decode_to_slice(&args.signing_key, &mut sk_bytes)?;
    let sk = AccountSigningKey::from_bytes(&sk_bytes);

    // build payload from Args
    let payload = build_payload(args)?;

    // build tx_intend
    let tx_intent = TxIntent::create(chain_id, addr, vk, payload)?;

    // build tx_envelope
    let tx_envelope = TxEnvelope::create(tx_intent, sk);
    let tx_envelope_wire = TxEnvelopeWire::from(&tx_envelope);
    Ok(tx_envelope_wire)
}

pub async fn send_envelope(envelope_wire: TxEnvelopeWire) -> anyhow::Result<()> {
    let client = Client::new();
    let response = client
        .post("http://localhost:8888/api/submit-tx")
        .header("content-type", "application/octet-stream")
        .body(envelope_wire.to_bcs_bytes())
        .send()
        .await?;
    tracing::info!("{:?}", response);
    tracing::info!("{}", response.text().await?);
    Ok(())
}

fn generate_payload(args: &TxArgs) -> anyhow::Result<TxPayload> {
    match args.payload_type.as_str() {
        "deploy" => {
            generate_deploy_payload(args)
        },
        "exec" => {
            generate_exec_payload(args)
        },
        _ => bail!("payload type mismatched")
    }
}

fn generate_deploy_payload(args: &TxArgs) -> anyhow::Result<TxPayload> {
    if let Some(json) = &args.deploy_json {
        let json_bytes = fs::read(json)?;
        let payload = serde_json::from_slice(&json_bytes)?;
        return Ok(payload)
    }
    if let Some(elf_path) = &args.deploy_elf {
        let elf_bytes = fs::read(elf_path)?;
        let elf_hash = sha256(&elf_bytes);
        tracing::info!("{:?}", elf_hash);
        let image_id = risc0_zkvm::compute_image_id(&elf_bytes)?;
        tracing::info!("{:?}", image_id);
        let payload = TxPayload::Deploy { source: elf_bytes };
        return Ok(payload)
    }
    bail!("deploy need at least one input")
}

fn generate_exec_payload(args: &TxArgs) -> anyhow::Result<TxPayload> {
    if let Some(json) = &args.exec_json {
    let bytes = fs::read(json)?;
    let payload = serde_json::from_slice(&bytes)?;
    return Ok(payload)
    }
    bail!("deploy need at least one input")
}