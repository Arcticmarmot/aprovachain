use anyhow::{ensure, Context, Result, anyhow};
use account::address::ContractAddress;
use apps::ctr_io::{CtrOutput, CtrResult};
use chain::block::Block;
use contract::contract::Contract;
use db::handle::DBHandle;
use tx::attestation::TxAttestation;
use tx::intent::TxPayload;

pub fn handle_tx_received(_: Vec<u8>) -> Result<()> {
    Ok(())
}

pub fn handle_block_received(block_bytes: Vec<u8>) -> Result<()> {
    // 解码 block
    let block = Block::try_decode_bcs(&block_bytes)?;
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
    for tx in &txs {
        // 检查节点签名
        tx.self_verify()?;
        // 检查用户签名
        let envelope = &tx.outcome.envelope;
        envelope.self_verify()?;
        let receipt_opt = tx.outcome.receipt_opt.clone();
        let intent = &envelope.intent;
        let payload = &envelope.intent.payload;

        match payload {
            TxPayload::Exec { ctr_addr_str, input, .. } => {
                let ctr_addr = ContractAddress::parse_bech32m_with_id(intent.chain_id, ctr_addr_str)?;
                let ctr_addr_bytes = ctr_addr.to_bytes();
                let ctr = db_handle.load_contract(&ctr_addr_bytes)?
                    .ok_or_else(|| anyhow!("invalid contract address"))?;
                let image_id = ctr.image_id;
                let receipt = receipt_opt.ok_or_else(|| anyhow!("exec tx must have receipt"))?;
                receipt.verify(image_id).context("verify receipt failed")?;
                // 简单的 ELF 文件， input == output
                let ctr_output_bytes: Vec<u8> = receipt.journal.decode().context("receipt decode failed")?;
                let ctr_output = CtrOutput::try_decode_bcs(&ctr_output_bytes).context("output decode failed")?;
                let input_hash = &ctr_output.input_hash;
                match &ctr_output.ctr_result {
                    CtrResult::Ok { outcome } => {
                        let mut is_valid = true;
                        let read_set = &outcome.effects.read_set;
                        let write_set = &outcome.effects.write_set;
                        for (ns_key, snap) in read_set {
                            match snap {
                                Some(snap) => {
                                    let read_ver = snap.version;
                                    match db_handle.load_data_entry(&ns_key)? {
                                        Some(curr_snap) => {
                                            if curr_snap.version > read_ver {
                                                is_valid = false;
                                                break;
                                            }
                                        },
                                        // 读集中有内容，数据库中已经删除
                                        None => {
                                            is_valid = false;
                                            break;
                                        }
                                    }
                                },
                                // 读集中没有读到内容，交易仍然成功执行，跳过检查
                                None => { }
                            }
                        }
                        // read_set 检查完毕，开始写入 write_set 到数据库
                        if is_valid {
                            // TODO: 写入应该是 Option<Vec<u8>> 类型，版本由 db_handle 决定，应用层不应该关注其内部细节
                            for (ns_key, snap_opt) in write_set {
                                match snap_opt {
                                    Some(snap) => {
                                        let val = &snap.value;
                                        db_handle.save_data_entry(ns_key, val.clone())?;
                                    },
                                    None => { }
                                }
                            }
                        } else {
                            tracing::info!(target: "node::event", tx_id=%tx.tx_id, "invalid tx")
                        }
                    },
                    CtrResult::Err { message } => {
                        tracing::error!(target: "node::event", %message, "ctr exec failed");
                    }
                };
                tracing::info!(target:"node::event", input=?input, output=?ctr_output);
            },
            TxPayload::Deploy { image_id, elf_hash, elf } => {
                let ctr = Contract::create(intent.chain_id, elf_hash, image_id,
                                           &envelope.verifying_key, intent.nonce);
                let ctr_addr_bytes = ctr.addr.to_bytes();
                // key: 合约的 addr 字节数组
                // value: 合约的BCS编码
                db_handle.save_contract(&ctr_addr_bytes, &ctr.to_canonical_bytes())?;
                // key: ELF文件哈希
                // value: ELF文件字节数组
                db_handle.save_elf(ctr.elf_hash, elf)?;
                tracing::info!(target:"node::event", contract_addr=?ctr.addr, "contract deployed");
            }
        }
    }

    // 存储 block
    db_handle.save_block(&block)?;

    // 更新 chain_state
    db_handle.save_chain_state(&header)?;

    // TODO: delete tracing info
    if let Some(block) = db_handle.load_block(header.height)? {
        tracing::info!(target: "node::block", ?block, "=======BLOCK=======\r\n");
    }
    Ok(())
}