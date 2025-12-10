use anyhow::{ensure, Context, Result};
use account::address::ContractAddress;
use account::executor::ExecutorId;
use account::keypair::AccountSigningKey;
use apps::ctr_io::{CtrInput, CtrOutput, CtrResult};
use chain::block::{OrderedBlock, LedgerBlock};
use contract::contract::Contract;
use db::handle::DBHandle;
use network::handle::P2pCmdHandle;
use platform::clock::unix_time_millis;
use primitives::constant::{SLOT_SECS};
use primitives::hash::sha256;
use schedule::dispatch::assign_executor_for_tx;
use server::pipeline::{build_tx_outcome};
use tx::attestation::TxAttestation;
use tx::code::{TxServiceCode, TxServiceCodeMap};
use tx::envelope::{TxEnvelope, TxEnvelopeWire};
use tx::intent::TxPayload;


pub fn handle_envelope_received(cmd_handle: P2pCmdHandle, sk: AccountSigningKey, envelope_bytes: Vec<u8>) -> Result<()> {
    // 加载状态信息
    let db_handle = DBHandle::new()?;
    let self_exec_id = ExecutorId(sk.verifying_key());

    // 从字节数组构造 TxEnvelope
    let wire: TxEnvelopeWire = TxEnvelopeWire::try_decode_bcs(envelope_bytes.as_ref())?;
    let envelope = TxEnvelope::try_from(wire)?;
    let tx_envelope_id = envelope.tx_id();
    // TODO: 重放交易攻击，拒绝重复的 nonce
    // 验证交易签名是否有效
    envelope.self_verify()?;
    tracing::info!(target:"node::event", ?tx_envelope_id, "tx envelope id");

    let exec_id = match assign_executor_for_tx(&db_handle, &tx_envelope_id)? {
        Some(exec_id) => { exec_id },
        None => {
            tracing::info!(target: "node::event", %tx_envelope_id, "no metrics yet, fall back to self as executor");
            self_exec_id
        }
    };
    tracing::info!(target:"node::event", %exec_id, "executor id");

    if exec_id == self_exec_id {
        tracing::info!(target:"node::event", "I will do it");
        // 执行交易
        let outcome = build_tx_outcome(&db_handle, envelope)?;

        let tx = TxAttestation::create(outcome, sk);
        let tx_bytes = tx.to_canonical_bytes();
        tracing::info!(target: "node::event", len=?tx_bytes.len(), "tx_size");

        // 广播交易
        cmd_handle.publish_tx(tx_bytes)?;
    } else {
        tracing::info!(target:"node::event", "none of my business");
    }
    Ok(())
}

pub fn handle_tx_received(_: Vec<u8>) -> Result<()> {
    Ok(())
}

pub fn handle_block_received(block_bytes: Vec<u8>) -> Result<()> {
    // 解码 block
    let block = OrderedBlock::try_decode_bcs(&block_bytes)?;
    let header = block.header;
    tracing::info!(target:"node::event", ?header, "new block header");
    let db_handle = DBHandle::new()?;
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

    // 每条 tx 的业务层校验交易
    let tx_codes = handle_tx(txs, &db_handle)?;
    let committed_block = LedgerBlock::new(block.header, block.txs, tx_codes);
    // 存储 block
    db_handle.save_block(&committed_block)?;

    // 更新 chain_state
    db_handle.save_chain_state(&header)?;

    // TODO: delete tracing info
    if let Some(block) = db_handle.load_block(header.height)? {
        tracing::info!(target: "node::block", ?block, "=======COMMITTED_BLOCK=======\r\n");
    }
    Ok(())
}

pub fn handle_tx(txs: Vec<TxAttestation>, db_handle: &DBHandle) -> Result<TxServiceCodeMap> {
    // tx_code 代表是否改变了世界状态
    let mut tx_codes: TxServiceCodeMap = TxServiceCodeMap::new();

    for tx in txs {
        let tx_id = tx.tx_id;
        let executor_id = ExecutorId(tx.verifying_key.clone());
        let code = apply_tx(db_handle, tx)?;
        tx_codes.insert(tx_id, (executor_id, code));
    }
    Ok(tx_codes)
}

pub fn apply_tx(db_handle: &DBHandle, tx: TxAttestation) -> Result<TxServiceCode> {
    // 检查节点签名
    if let Err(err) = tx.self_verify() {
        tracing::error!(target: "node::event", %err, "invalid node signature");
        return Ok(TxServiceCode::InvalidTx)
    }

    let outcome = tx.outcome;
    // 检查用户签名
    let envelope = outcome.envelope;

    if let Err(err) = envelope.self_verify() {
        tracing::error!(target: "node::event", %err, "invalid user signature");
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
                    tracing::error!(target:"node::event", %err, ctr_addr_str, "invalid ctr_addr_str for exec tx");
                    return Ok(TxServiceCode::InvalidTx);
                }
            };
            let ctr_addr_bytes = ctr_addr.to_bytes();
            let ctr_opt = db_handle.load_contract(&ctr_addr_bytes)?;
            let ctr = match ctr_opt {
                Some(ctr) => ctr,
                None => {
                    tracing::error!(target:"node::event", contract_addr=?ctr_addr, "contract not found");
                    return Ok(TxServiceCode::InvalidTx);
                }
            };

            let image_id = ctr.image_id;
            let receipt = match receipt_opt {
                Some(receipt) => receipt,
                None => {
                    tracing::error!(target:"node::event", "exec tx without receipt");
                    return Ok(TxServiceCode::InvalidTx);
                }
            };
            if let Err(err) = receipt.verify(image_id) {
                tracing::error!(target:"node::event", %err, "fake receipt");
                return Ok(TxServiceCode::FakeReceipt);
            }

            let ctr_output_bytes: Vec<u8> = match receipt.journal.decode() {
                Ok(bytes) => bytes,
                Err(err) => {
                    tracing::error!(target:"node::event", %err, "receipt journal decode failed");
                    return Ok(TxServiceCode::InvalidTx);
                }
            };

            let ctr_output = match CtrOutput::try_decode_bcs(&ctr_output_bytes) {
                Ok(output) => output,
                Err(err) => {
                    tracing::error!(target:"node::event", %err, "output decode failed");
                    return Ok(TxServiceCode::InvalidTx);
                }
            };
            tracing::info!(target:"node::event", input=?input, output=?ctr_output);

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
                    tracing::error!(target: "node::event", %message, "ctr exec failed");
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
            tracing::info!(target:"node::event", contract_addr=?ctr.addr, "contract deployed");

            Ok(TxServiceCode::Success)
        },
        TxPayload::Update { ctr_addr_str,  image_id, elf_hash, elf } => {
            let ctr = Contract::create(intent.chain_id, &image_id, &elf_hash,
                                       &envelope.verifying_key, intent.nonce);
            // ctr_addr 保持不变，由传入的决定
            let ctr_addr = match ContractAddress::parse_bech32m_with_id(intent.chain_id, &ctr_addr_str) {
                Ok(addr) => addr,
                Err(err) => {
                    tracing::error!(target:"node::event", %err, ctr_addr_str, "invalid ctr_addr_str for update tx");
                    return Ok(TxServiceCode::InvalidTx);
                }
            };

            // key: 合约的 addr 字节数组
            // value: 合约的BCS编码
            db_handle.save_contract(&ctr_addr.to_bytes(), &ctr.to_canonical_bytes())?;
            // key: ELF文件哈希
            // value: ELF文件字节数组
            db_handle.save_elf(ctr.elf_hash, &elf)?;
            tracing::info!(target:"node::event", contract_addr=?ctr_addr, "contract update");

            Ok(TxServiceCode::Success)
        },
    }
}