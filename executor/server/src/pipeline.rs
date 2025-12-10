use apps::ctr_io::{CtrOutput, CtrResult};
use contract::contract::Contract;
use tx::intent::TxPayload;
use tx::outcome::TxOutcome;
use crate::context::SubmitTxResponse;
use crate::error::{ServerError,Result};

pub fn resp_from_outcome(outcome: &TxOutcome) -> Result<SubmitTxResponse> {
    let payload = &outcome.envelope.intent.payload;
    match payload {
        TxPayload::Exec { ctr_addr_str, input, access_set } => {
            let receipt = outcome.receipt_opt.as_ref().ok_or_else(|| ServerError::ReceiptNotFound)?;
            let ctr_output_bytes: Vec<u8> = receipt.journal.decode()?;
            let ctr_output = CtrOutput::try_decode_bcs(&ctr_output_bytes)?;
            tracing::info!(target: "node::server", ?ctr_output);
            match ctr_output.ctr_result {
                CtrResult::Ok { outcome } => {
                    Ok(SubmitTxResponse::Exec {
                        ctr_addr_str: ctr_addr_str.clone(),
                        input: input.clone(),
                        access_set: access_set.clone(),
                        receipt: receipt.clone(),
                        answer: outcome.answer
                    })
                },
                CtrResult::Err { message } => {
                    Err(ServerError::ContractExec { message })
                }
            }
        },
        TxPayload::Deploy  { image_id, elf_hash, .. } => {
            let envelope = &outcome.envelope;
            let vk = &envelope.verifying_key;
            let intent = &envelope.intent;
            let chain_id = intent.chain_id;
            let nonce = intent.nonce;
            let ctr = Contract::create(chain_id, image_id, elf_hash, vk, nonce);
            Ok(SubmitTxResponse::Deploy {
                ctr_addr_str: ctr.addr.to_bech32m()?,
                image_id: image_id.clone(),
                elf_hash: elf_hash.clone(),
            })
        },
        TxPayload::Update  { ctr_addr_str, image_id, elf_hash, .. } => {
            Ok(SubmitTxResponse::Update {
                ctr_addr_str: ctr_addr_str.clone(),
                image_id: image_id.clone(),
                elf_hash: elf_hash.clone(),
            })
        }
    }
}
