use std::sync::Arc;
use std::time::Instant;
use anyhow::{ensure, Context, Result};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use account::address::{ChainAddrBytes, ContractAddress};
use account::executor::ExecutorId;
use apps::ctr_io::{CtrInput, CtrOutput, CtrResult, ReadSet, WriteSet};
use chain::block::{OrderedBlock, LedgerBlock};
use chain::catalog::{TxServiceCatalog, TxServiceCode};
use contract::contract::Contract;
use db::handle::DBHandle;
use primitives::hash::{sha256, Hash32};
use schedule::dispatch::assign_executor_for_tx;
use tx::attestation::TxAttestation;
use tx::id::TxAttestationId;
use tx::intent::TxPayload;

pub struct VerifyReport {
    idx: usize,
    tx_id : TxAttestationId,
    executor_id: ExecutorId,
    outcome: VerifyOutcome,
}

pub enum VerifyOutcome {
    Accept(ApplyInfo),
    Reject(TxServiceCode)
}

pub enum ApplyInfo {
    Exec {
        read_set: ReadSet,
        write_set: WriteSet,
        send_height: u128,
        slot_range: u128,
    },
    Deploy {
        ctr_addr_bytes: ChainAddrBytes,
        ctr_bytes: Vec<u8>,
        elf_hash: Hash32,
        elf_bytes: Vec<u8>,
    },
    Update {
        ctr_addr_bytes: ChainAddrBytes,
        ctr_bytes: Vec<u8>,
        elf_hash: Hash32,
        elf_bytes: Vec<u8>,
    }
}

pub async fn verify_and_apply_block(db_handle: &DBHandle, block_bytes: Vec<u8>) -> Result<()> {
    let validate_time = Instant::now();
    // 解码 block
    let block = OrderedBlock::try_decode_bcs(&block_bytes)?;
    let header = block.header;
    tracing::info!(target:"engine::verify", ?header, "new block header");
    // 验证是否是合法区块，并存储 chain_state
    match db_handle.load_chain_state()? {
        Some(tip_header) => {
            ensure!(header.height == tip_header.height + 1, "invalid new block header");
            ensure!(header.parent_hash == tip_header.hash(), "invalid new block header");
        },
        None => {
            // NOTE: 主网需要保证从第 0 个区块开始存储
            // ensure!(header.height == 0, "invalid new block header");
        }
    }

    // 验证区块内交易
    // 解码各个交易
    let txs: Vec<TxAttestation> = block.txs.iter()
        .cloned()
        .map(|tx| {
            TxAttestation::try_from(tx).with_context(|| "tx decode failed")
        })
        .collect::<Result<Vec<TxAttestation>>>()?;
    tracing::info!(target:"engine::verify", tx_num=txs.len(), "tx num");


    // Stage A: 并行 verify（重 CPU + 只读 DB + journal decode）
    let mut verify_js: JoinSet<Result<VerifyReport>> = JoinSet::new();
    let verify_workers = 16;
    let verify_sem = Arc::new(Semaphore::new(verify_workers));

    for (idx, tx) in txs.into_iter().enumerate() {
        let tx_db_handle = db_handle.clone();
        let curr_height = header.height;
        let sem = verify_sem.clone();
        verify_js.spawn(async move {
            let _permit = sem.acquire_owned().await.unwrap();
            verify_tx(&tx_db_handle, idx, tx, curr_height)
        });
    }

    let mut reports: Vec<VerifyReport> = Vec::new();
    while let Some(entry) = verify_js.join_next().await {
        let report = entry??;
        reports.push(report);
    }

    // 恢复块内确定顺序（非常关键）
    reports.sort_by_key(|r| r.idx);

    // Stage B: 串行 apply（确定性写入）
    let mut catalog = TxServiceCatalog::new();
    for report in reports {
        let tx_id = report.tx_id;
        let executor_id = report.executor_id;

        let code = match report.outcome {
            VerifyOutcome::Reject(code) => code,
            VerifyOutcome::Accept(apply_info) => apply_tx(db_handle, apply_info, header.height)?,
        };

        catalog.insert(tx_id, (executor_id, code));
    }

    // 每条 tx 的业务层校验交易
    let ledger_block = LedgerBlock::new(block, catalog);
    // 存储 block
    db_handle.save_block(&ledger_block)?;

    // 更新 chain_state
    db_handle.save_chain_state(&header)?;

    let elapsed = Instant::now().saturating_duration_since(validate_time);
    tracing::info!(target: "engine::execute", ?elapsed, "validate time: ");

    // TODO: delete the tracing info
    if let Some(block) = db_handle.load_block(header.height)? {
        tracing::info!(target: "executor::block", ?block, "=======BLOCK=======\r\n");
    }

    Ok(())
}

pub fn apply_tx(db_handle: &DBHandle, apply_info: ApplyInfo, curr_height: u128) -> Result<TxServiceCode> {
    match apply_info {
        ApplyInfo::Exec { read_set, write_set, send_height, slot_range } => {
            // 先 timeout：避免超时交易写状态
            if send_height + slot_range < curr_height {
                return Ok(TxServiceCode::Timeout);
            }

            let is_conflict = db_handle.apply_rw_set(&read_set, &write_set)?;
            if is_conflict {
                Ok(TxServiceCode::Conflict)
            } else {
                Ok(TxServiceCode::Success)
            }
        }

        ApplyInfo::Deploy { ctr_addr_bytes, ctr_bytes, elf_hash, elf_bytes } => {
            db_handle.save_contract(&ctr_addr_bytes, &ctr_bytes)?;
            db_handle.save_elf(elf_hash, &elf_bytes)?;
            Ok(TxServiceCode::Success)
        }

        ApplyInfo::Update { ctr_addr_bytes, ctr_bytes, elf_hash, elf_bytes } => {
            db_handle.save_contract(&ctr_addr_bytes, &ctr_bytes)?;
            db_handle.save_elf(elf_hash, &elf_bytes)?;
            Ok(TxServiceCode::Success)
        }
    }
}
pub fn verify_tx(db_handle: &DBHandle, idx: usize, tx: TxAttestation, curr_height: u128) -> Result<VerifyReport> {
    let tx_id = tx.tx_id;
    let executor_id = ExecutorId(tx.verifying_key.clone());
    // helper：快速返回 Reject
    let reject = |code: TxServiceCode| -> VerifyReport {
        VerifyReport { idx, tx_id, executor_id, outcome: VerifyOutcome::Reject(code) }
    };
    // helper：快速返回 Accept
    let accept = |apply_info: ApplyInfo| -> VerifyReport {
        VerifyReport { idx, tx_id, executor_id, outcome: VerifyOutcome::Accept(apply_info) }
    };

    // executor signature
    if let Err(err) = tx.self_verify() {
        tracing::error!(target: "engine::verify", %err, "invalid executor signature");
        return Ok(reject(TxServiceCode::InvalidTx));
    }

    let outcome = tx.outcome;
    let envelope = outcome.envelope;

    // user signature
    if let Err(err) = envelope.self_verify() {
        tracing::error!(target: "engine::verify", %err, "invalid user signature");
        return Ok(reject(TxServiceCode::InvalidTx));
    }

    let receipt_opt = outcome.receipt_opt;
    let intent = &envelope.intent;

    match &intent.payload {
        TxPayload::Exec { ctr_addr_str, input, .. } => {
            // schedule 指派校验（只读）
            let assign_time = Instant::now();

            let envelope_id = &envelope.tx_id();
            match assign_executor_for_tx(db_handle, envelope_id, intent.timestamp)? {
                Some(expect_exec_id) => {
                    if expect_exec_id.verifying_key() != tx.verifying_key {
                        tracing::error!(target: "engine::verify", %expect_exec_id, schedule_exec_id=%ExecutorId(tx.verifying_key));
                        return Ok(reject(TxServiceCode::InvalidTx));
                    }
                }
                None => {}
            }
            let assign_elapsed = Instant::now().saturating_duration_since(assign_time);
            tracing::info!(target: "engine::execute", ?assign_elapsed, "verify time: ");

            // load contract -> image_id（只读）
            let ctr_addr = match ContractAddress::parse_bech32m_with_id(intent.chain_id, ctr_addr_str) {
                Ok(addr) => addr,
                Err(err) => {
                    tracing::error!(target:"engine::verify", %err, ctr_addr_str, "invalid ctr_addr_str for exec tx");
                    return Ok(reject(TxServiceCode::InvalidTx));
                }
            };

            let ctr = match db_handle.load_contract(&ctr_addr.to_bytes())? {
                Some(ctr) => ctr,
                None => {
                    tracing::error!(target:"engine::verify", contract_addr=?ctr_addr, "contract not found");
                    return Ok(reject(TxServiceCode::InvalidTx));
                }
            };
            let image_id = ctr.image_id;

            // receipt
            let receipt = match receipt_opt {
                Some(receipt) => receipt,
                None => {
                    tracing::error!(target:"engine::verify", "exec tx without receipt");
                    return Ok(reject(TxServiceCode::InvalidTx));
                }
            };

            // proof verify（最重）
            let verify_time = Instant::now();
            if let Err(err) = receipt.verify(image_id) {
                tracing::error!(target:"engine::verify", %err, "fake receipt");
                return Ok(reject(TxServiceCode::FakeReceipt));
            }
            let verify_elapsed = Instant::now().saturating_duration_since(verify_time);
            tracing::info!(target: "engine::execute", ?verify_elapsed, "verify time: ");

            // decode journal -> CtrOutput（只做一次）
            let ctr_output_bytes: Vec<u8> = match receipt.journal.decode() {
                Ok(bytes) => bytes,
                Err(err) => {
                    tracing::error!(target:"engine::verify", %err, "receipt journal decode failed");
                    return Ok(reject(TxServiceCode::InvalidTx));
                }
            };
            let ctr_output = match CtrOutput::try_decode_bcs(&ctr_output_bytes) {
                Ok(output) => output,
                Err(err) => {
                    tracing::error!(target:"engine::verify", %err, "output decode failed");
                    return Ok(reject(TxServiceCode::InvalidTx));
                }
            };

            // input binding
            let input_hash = ctr_output.input_hash;
            let read_set = ctr_output.read_set;

            let ctr_input = CtrInput {
                chain_id: intent.chain_id,
                input: input.clone(),
                read_set: read_set.clone(),
            };
            if input_hash != sha256(ctr_input.encode_bcs()) {
                return Ok(reject(TxServiceCode::FakeInput));
            }

            // 提取 write_set（apply 阶段不再 decode）
            let write_set = match &ctr_output.ctr_result {
                CtrResult::Ok { outcome } => outcome.write_set.clone(),
                CtrResult::Err { message } => {
                    tracing::error!(target: "engine::verify", %message, "ctr exec failed");
                    return Ok(reject(TxServiceCode::BadRequest));
                }
            };

            // timeout 元信息（只读）
            let slot_range = intent.scale.to_slot_count() as u128;
            let send_height = match db_handle.load_ts_height(intent.timestamp)? {
                Some(h) => h,
                None => return Ok(reject(TxServiceCode::InvalidTx)),
            };

            Ok(accept(ApplyInfo::Exec {
                read_set,
                write_set,
                send_height,
                slot_range,
            }))
        }

        // 注意：Deploy/Update 在 verify 阶段只构造 bytes，不写 DB
        TxPayload::Deploy { image_id, elf_hash, elf } => {
            let ctr = Contract::create(
                intent.chain_id,
                image_id,
                elf_hash,
                &envelope.verifying_key,
                intent.nonce,
            );

            Ok(accept(ApplyInfo::Deploy {
                ctr_addr_bytes: ctr.addr.to_bytes(),
                ctr_bytes: ctr.to_canonical_bytes(),
                elf_hash: ctr.elf_hash,
                elf_bytes: elf.clone(),
            }))
        }

        TxPayload::Update { ctr_addr_str, image_id, elf_hash, elf } => {
            let ctr = Contract::create(
                intent.chain_id,
                image_id,
                elf_hash,
                &envelope.verifying_key,
                intent.nonce,
            );

            let ctr_addr = match ContractAddress::parse_bech32m_with_id(intent.chain_id, ctr_addr_str) {
                Ok(addr) => addr,
                Err(err) => {
                    tracing::error!(target:"engine::verify", %err, ctr_addr_str, "invalid ctr_addr_str for update tx");
                    return Ok(reject(TxServiceCode::InvalidTx));
                }
            };

            Ok(accept(ApplyInfo::Update {
                ctr_addr_bytes: ctr_addr.to_bytes(),
                ctr_bytes: ctr.to_canonical_bytes(),
                elf_hash: ctr.elf_hash,
                elf_bytes: elf.clone(),
            }))
        }
    }
}