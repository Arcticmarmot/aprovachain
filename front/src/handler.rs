use std::fs;
use std::path::PathBuf;
use anyhow::{anyhow, bail};
use clap::{Parser};
use reqwest::{Client, Response};
use account::keypair::{AccountSigningKey, AccountSigningKeyBytes};
use platform::file::{load_user_sk_path};
use spec::chain::ChainId;
use primitives::hash::sha256;
use tx::envelope::{TxEnvelope, TxEnvelopeWire};
use tx::intent::{TxIntent, TxPayload, TxScale};

#[derive(Debug)]
pub struct TxBuildSpec {
    chain_id: ChainId,
    sk: AccountSigningKey,
    scale: TxScale,
    payload: TxPayload
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
pub struct TxArgs {
    #[clap(long, env, next_help_heading = "The Chain Id of the Tx")]
    pub chain_id: u64,

    #[clap(long, env, next_help_heading = "The Chain Id of the Tx")]
    pub scale: u32,

    #[clap(long, env, next_help_heading = "The payload type of TxPayload")]
    pub payload_type: String,

    #[clap(long, env, next_help_heading = "The payload type of TxPayload")]
    pub ctr_addr_str: Option<String>,

    #[clap(long, env, next_help_heading = "The deploy json of TxPayload")]
    pub deploy_json: Option<PathBuf>,

    #[clap(long, env, next_help_heading = "The deploy elf of TxPayload")]
    pub deploy_elf: Option<PathBuf>,

    #[clap(long, env, next_help_heading = "The update elf of TxPayload")]
    pub update_elf: Option<PathBuf>,

    #[clap(long, env, next_help_heading = "The exec json of TxPayload")]
    pub exec_json: Option<PathBuf>,
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

    // build scale from Args
    let scale = TxScale::try_from(args.scale)?;

    // build sk from Args
    let sk_bytes = load_user_sk_bytes()?;
    let sk = AccountSigningKey::from_bytes(&sk_bytes);

    // build payload from Args
    let payload = build_payload(args)?;

    Ok(TxBuildSpec {
        chain_id,
        sk,
        scale,
        payload
    })
}

pub fn create_build_spec(chain_id: ChainId, scale: u32, payload: TxPayload) -> anyhow::Result<TxBuildSpec> {
    // build sk from Args
    let sk_bytes = load_user_sk_bytes()?;
    let sk = AccountSigningKey::from_bytes(&sk_bytes);
    let scale = TxScale::try_from(scale)?;

    Ok(TxBuildSpec {
        chain_id,
        sk,
        scale,
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
    let tx_intent = TxIntent::create(spec.chain_id, spec.scale, spec.payload)?;
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
        "Exec" => {
            generate_exec_payload(args)
        },
        "Deploy" => {
            generate_deploy_payload(args)
        },
        "Update" => {
            generate_update_payload(args)
        },
        _ => bail!("payload type mismatched")
    }
}

fn generate_exec_payload(args: &TxArgs) -> anyhow::Result<TxPayload> {
    if let Some(json) = &args.exec_json {
        let exec_bytes = fs::read(json)?;
        let payload = serde_json::from_slice(&exec_bytes)?;
        return Ok(payload)
    }
    bail!("deploy need at least one input")
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


fn generate_update_payload(args: &TxArgs) -> anyhow::Result<TxPayload> {
    let ctr_addr_str = args.ctr_addr_str.as_ref().ok_or_else(|| anyhow!("must have ctr_addr_str"))?;
    if let Some(elf_path) = &args.update_elf {
        let elf_bytes = fs::read(elf_path)?;
        let image_id = risc0_zkvm::compute_image_id(&elf_bytes)?;
        tracing::info!("{:?}", image_id);
        let elf_hash = sha256(&elf_bytes);
        tracing::info!("{:?}", elf_hash);
        let payload = TxPayload::Update {
            ctr_addr_str: ctr_addr_str.clone(),
            image_id,
            elf: elf_bytes,
            elf_hash
        };
        return Ok(payload)
    }
    bail!("deploy need at least one input")
}