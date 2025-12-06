use tx::envelope::{TxEnvelopeWire, TxEnvelope};
use axum::body::{Bytes};
use axum::extract::State;
use axum::Json;
use risc0_zkvm::{default_prover, Digest, ExecutorEnv, ProverOpts, Prover, Receipt};
use tx::intent::{TxPayload};
use crate::error::{ApiResult, ServerError, Result};
use primitives::hash::{sha256, Hash32};
use account::address::{ContractAddress};
use contract::contract::{Contract};
use db::handle::DBHandle;
use apps::ctr_io::{AccessSet, CtrInput, CtrOutput, CtrResult, ReadSet};
use tx::outcome::{TxOutcome};
use tx::attestation::TxAttestation;
use crate::context::{AppState, SubmitTxResponse};
use crate::error::ServerError::ProofGenerate;
use crate::pipeline::build_tx_outcome;

/// 交易提交处理函数
pub async fn submit_tx(State(state) : State<AppState>, tx_bytes: Bytes) -> ApiResult<SubmitTxResponse> {
    // 加载状态信息
    let db_handle = state.db_handle;
    let cmd_handle = state.cmd_handle;
    let sk = state.sk;
    // 从字节数组构造 TxEnvelope
    let wire: TxEnvelopeWire = TxEnvelopeWire::try_decode_bcs(tx_bytes.as_ref())?;
    let envelope = TxEnvelope::try_from(wire)?;

    // TODO: 重放交易攻击，拒绝重复的 nonce
    // 验证交易签名是否有效
    envelope.self_verify()?;
    tracing::info!(target:"node::server", tx_envelope_id=%envelope.tx_id(), "tx envelope id");

    // 执行交易
    let outcome = build_tx_outcome(&db_handle, envelope)?;
    let response = resp_from_outcome(&outcome)?;

    let tx = TxAttestation::create(outcome, sk);
    let tx_bytes = tx.to_canonical_bytes();
    tracing::info!(target: "node::server", len=?tx_bytes.len(), "tx_size");

    // 广播交易
    cmd_handle.publish_tx(tx_bytes)?;

    // 返回 response
    Ok(Json(response))
}

pub fn resp_from_outcome(outcome: &TxOutcome) -> Result<SubmitTxResponse> {
    let payload = &outcome.envelope.intent.payload;
    match payload {
        TxPayload::Exec { ctr_addr_str, input, access_set } => {
            let receipt = &outcome.receipt_opt;
            match receipt {
                Some(receipt) => {
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
                None => {
                    Err(ServerError::ReceiptNotFound)
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
