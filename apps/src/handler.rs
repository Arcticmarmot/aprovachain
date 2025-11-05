use std::fs;
use clap::Parser;
use reqwest::Client;
use account::address::ChainAddress;
use account::keypair::{AccountSigningKey, AccountVerifyingKey};
use chain::spec::ChainId;
use contracts::{UAV_ELF, UAV_ID};
use tx::tx_envelope::{TxEnvelope, TxEnvelopeWire};
use tx::tx_intent::{TxIntent, TxPayload};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
pub struct TxArgs {
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
pub fn parse_tx_args(args: TxArgs) -> anyhow::Result<TxEnvelopeWire> {
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

    // TODO: 读入 ELF 文件
    // read the payload file
    // build payload from payload file
    let bytes = fs::read(args.payload_path)?;
    let payload: TxPayload = serde_json::from_slice(&bytes)?;


    let elf_file = UAV_ELF;
    let elf_id = UAV_ID;
    tracing::info!("{:?}", elf_id);
    tracing::info!("LEN: {}", elf_file.len());
    // let payload = TxPayload::Deploy { source: Vec::from(elf_file) };

    // let bin = read_bin_file("/home/woolf/aprova/aprova/apps/ELF/aprova-guest.bin").unwrap();
    // let payload = TxPayload::Deploy { source: bin };
    // println!("{:?}", payload);

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
    println!("{:?}", response);
    println!("{}", response.text().await?);
    Ok(())
}