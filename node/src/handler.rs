use tx::tx_envelope::{TxEnvelopeWire, TxEnvelope};
use axum::body::Bytes;
use risc0_zkvm::{default_prover, ExecutorEnv, Prover};
use db::controller::{kv_get, kv_put};
use tx::tx_intent::{TxIntent, TxPayload};
use crate::error::{Result};
use primitives::hash::sha256;

/// 交易提交处理函数
pub async fn submit_tx(tx_bytes: Bytes) -> Result<String> {
    let tx_envelope_wire: TxEnvelopeWire = TxEnvelopeWire::try_from_bcs_bytes(tx_bytes.as_ref())?;
    let tx_envelope = TxEnvelope::try_from(tx_envelope_wire)?;
    // println!("{:?}", tx_envelope);
    let _ = tx_handler(&tx_envelope.intent);
    let verify_result = verify_tx_sig(&tx_envelope);
    match verify_result {
        Ok(()) => Ok("submitted and verified success".to_string()),
        Err(e) => Ok("invalid transaction".to_string())
    }
}

/// 验证交易签名
pub fn verify_tx_sig(envelope: &TxEnvelope) -> Result<()> {
    let vk = &envelope.intent.verifying_key;
    let intent_id_hash = envelope.intent.tx_intent_id().0;
    vk.verify(&intent_id_hash, &envelope.signature)?;
    Ok(())
}

pub fn tx_handler(intent: &TxIntent) -> Result<()> {
    let payload = &intent.payload;
    match payload {
        TxPayload::Deploy{ source } => {
            let image_id = sha256(source);
            println!("{:?}", image_id);
            kv_put(&image_id, source)?;
        },
        TxPayload::Exec { image_id, input} => {
            let env = ExecutorEnv::builder()
                .write(&input)
                .unwrap()
                .build()
                .unwrap();

            let prover = default_prover();
            let elf_file = match kv_get(image_id)?{
                Some(elf) => elf,
                None => return Ok(())
            };

            println!("LEN: {}", elf_file.len());
            let proof = prover.prove(env, &elf_file);
            println!("PROOF: {:?}", proof);
        }
    }
    Ok(())
}
///rocksdb init success...
// node listening on http(s)://0.0.0.0:8888 ...
// [87, 180, 127, 63, 32, 217, 242, 144, 114, 72, 91, 191, 217, 216, 167, 155, 210, 124, 178, 39, 99, 157, 18, 116, 116, 46, 160, 36, 59, 49, 142, 224]
// LEN: 336824
// PROOF: Ok(ProveInfo { receipt: Receipt { inner: Composite(CompositeReceipt { segments: 1 segments, assumption_receipts: 0 assumptions, verifier_parameters: Digest(4bce006e0858edf3a3726987c0b1b6258224c000971e451bc9c05cfec086a84b) }), journal: Journal { bytes: 20 bytes }, metadata: ReceiptMetadata { verifier_parameters: Digest(4bce006e0858edf3a3726987c0b1b6258224c000971e451bc9c05cfec086a84b) } }, stats: SessionStats { segments: 1, total_cycles: 32768, user_cycles: 4599, paging_cycles: 17874, reserved_cycles: 10295 }, work_receipt: None })