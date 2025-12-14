use anyhow::{ensure, Context, Result};
use account::address::ContractAddress;
use account::executor::ExecutorId;
use apps::ctr_io::{CtrInput, CtrOutput, CtrResult};
use chain::block::{OrderedBlock, LedgerBlock};
use chain::catalog::{TxServiceCatalog, TxServiceCode};
use contract::contract::Contract;
use db::handle::DBHandle;
use platform::clock::unix_time_millis;
use primitives::constant::{SLOT_SECS};
use primitives::hash::sha256;
use schedule::dispatch::assign_executor_for_tx;
use tx::attestation::TxAttestation;
use tx::intent::TxPayload;

pub fn verify_and_apply_block(db_handle: &DBHandle, block_bytes: Vec<u8>) -> Result<()> {
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
    let mut catalog = TxServiceCatalog::new();

    for tx in txs {
        let tx_id = tx.tx_id;
        let exec_id = ExecutorId(tx.verifying_key.clone());
        let code = verify_and_apply_tx(db_handle, tx)?;
        catalog.insert(tx_id, (exec_id, code));
    }
    
    // 每条 tx 的业务层校验交易
    let ledger_block = LedgerBlock::new(block, catalog);
    // 存储 block
    db_handle.save_block(&ledger_block)?;

    // 更新 chain_state
    db_handle.save_chain_state(&header)?;

    // TODO: delete the tracing info
    if let Some(block) = db_handle.load_block(header.height)? {
        tracing::info!(target: "executor::block", ?block, "=======BLOCK=======\r\n");
    }
    Ok(())
}

pub fn verify_and_apply_tx(db_handle: &DBHandle, tx: TxAttestation) -> Result<TxServiceCode> {
    // 检查节点签名
    if let Err(err) = tx.self_verify() {
        tracing::error!(target: "engine::verify", %err, "invalid executor signature");
        return Ok(TxServiceCode::InvalidTx)
    }

    let outcome = tx.outcome;
    // 检查用户签名
    let envelope = outcome.envelope;
    // 检查是否为 schedule 指定节点执行
    let exec_id_opt = assign_executor_for_tx(&db_handle, &envelope.tx_id(), envelope.intent.timestamp)?;
    match exec_id_opt {
        Some(exec_id) => {
            if exec_id.verifying_key() != tx.verifying_key {
                tracing::error!(target: "engine::verify", %exec_id, schedule_exec_id=%ExecutorId(tx.verifying_key));
                return Ok(TxServiceCode::InvalidTx)
            }
        }
        None => { }
    }
    
    if let Err(err) = envelope.self_verify() {
        tracing::error!(target: "engine::verify", %err, "invalid user signature");
        return Ok(TxServiceCode::InvalidTx)
    }

    let receipt_opt = outcome.receipt_opt;
    let intent = envelope.intent;
    let payload = intent.payload;
    
    match payload {
        TxPayload::Exec { ctr_addr_str, input, .. } => {
            let ctr_addr = match ContractAddress::parse_bech32m_with_id(intent.chain_id, &ctr_addr_str) {
                Ok(addr) => addr,
                Err(err) => {
                    tracing::error!(target:"engine::verify", %err, ctr_addr_str, "invalid ctr_addr_str for exec tx");
                    return Ok(TxServiceCode::InvalidTx);
                }
            };
            let ctr_addr_bytes = ctr_addr.to_bytes();
            let ctr_opt = db_handle.load_contract(&ctr_addr_bytes)?;
            let ctr = match ctr_opt {
                Some(ctr) => ctr,
                None => {
                    tracing::error!(target:"engine::verify", contract_addr=?ctr_addr, "contract not found");
                    return Ok(TxServiceCode::InvalidTx);
                }
            };

            let image_id = ctr.image_id;
            let receipt = match receipt_opt {
                Some(receipt) => receipt,
                None => {
                    tracing::error!(target:"engine::verify", "exec tx without receipt");
                    return Ok(TxServiceCode::InvalidTx);
                }
            };
            if let Err(err) = receipt.verify(image_id) {
                tracing::error!(target:"engine::verify", %err, "fake receipt");
                return Ok(TxServiceCode::FakeReceipt);
            }

            let ctr_output_bytes: Vec<u8> = match receipt.journal.decode() {
                Ok(bytes) => bytes,
                Err(err) => {
                    tracing::error!(target:"engine::verify", %err, "receipt journal decode failed");
                    return Ok(TxServiceCode::InvalidTx);
                }
            };

            let ctr_output = match CtrOutput::try_decode_bcs(&ctr_output_bytes) {
                Ok(output) => output,
                Err(err) => {
                    tracing::error!(target:"engine::verify", %err, "output decode failed");
                    return Ok(TxServiceCode::InvalidTx);
                }
            };
            tracing::info!(target:"engine::verify", input=?input, output=?ctr_output);

            let input_hash = ctr_output.input_hash;
            let read_set = ctr_output.read_set;
            let ctr_input = CtrInput {
                chain_id: intent.chain_id,
                input,
                read_set: read_set.clone()
            };

            if input_hash != sha256(ctr_input.encode_bcs()) {
                return Ok(TxServiceCode::FakeInput);
            }

            match &ctr_output.ctr_result {
                CtrResult::Ok { outcome } => {
                    let is_conflict = db_handle.apply_rw_set(&read_set, &outcome.write_set)?;
                    if is_conflict {
                        return Ok(TxServiceCode::Conflict);
                    }
                },
                CtrResult::Err { message } => {
                    tracing::error!(target: "engine::verify", %message, "ctr exec failed");
                    return Ok(TxServiceCode::BadRequest);
                }
            };
            // Timeout 判断
            let now = unix_time_millis()?;
            let elapsed = now - intent.timestamp;
            let allow_elapsed: u128 = SLOT_SECS as u128 * intent.scale.to_slot_count() as u128 * 1000;

            if elapsed > allow_elapsed {
                Ok(TxServiceCode::Timeout)
            } else {
                Ok(TxServiceCode::Success)
            }
        },
        TxPayload::Deploy { image_id, elf_hash, elf } => {
            let ctr = Contract::create(intent.chain_id, &image_id, &elf_hash,
                                       &envelope.verifying_key, intent.nonce);
            let ctr_addr_bytes = ctr.addr.to_bytes();
            // key: 合约的 addr 字节数组
            // value: 合约的BCS编码
            db_handle.save_contract(&ctr_addr_bytes, &ctr.to_canonical_bytes())?;
            // key: ELF文件哈希
            // value: ELF文件字节数组
            db_handle.save_elf(ctr.elf_hash, &elf)?;
            tracing::info!(target:"engine::verify", contract_addr=?ctr.addr, "contract deployed");

            Ok(TxServiceCode::Success)
        },
        TxPayload::Update { ctr_addr_str,  image_id, elf_hash, elf } => {
            let ctr = Contract::create(intent.chain_id, &image_id, &elf_hash,
                                       &envelope.verifying_key, intent.nonce);
            // ctr_addr 保持不变，由传入的决定
            let ctr_addr = match ContractAddress::parse_bech32m_with_id(intent.chain_id, &ctr_addr_str) {
                Ok(addr) => addr,
                Err(err) => {
                    tracing::error!(target:"engine::verify", %err, ctr_addr_str, "invalid ctr_addr_str for update tx");
                    return Ok(TxServiceCode::InvalidTx);
                }
            };

            // key: 合约的 addr 字节数组
            // value: 合约的BCS编码
            db_handle.save_contract(&ctr_addr.to_bytes(), &ctr.to_canonical_bytes())?;
            // key: ELF文件哈希
            // value: ELF文件字节数组
            db_handle.save_elf(ctr.elf_hash, &elf)?;
            tracing::info!(target:"engine::verify", contract_addr=?ctr_addr, "contract update");

            Ok(TxServiceCode::Success)
        },
    }
}