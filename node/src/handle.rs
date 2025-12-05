use anyhow::{ensure, Context, Result, anyhow};
use account::address::ContractAddress;
use apps::ctr_io::{CtrContext, CtrInput, CtrOutput, CtrResult};
use chain::block::{OrderedBlock, LedgerBlock};
use contract::contract::Contract;
use db::handle::DBHandle;
use primitives::hash::sha256;
use tx::attestation::TxAttestation;
use tx::intent::TxPayload;

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

pub fn handle_tx(txs: Vec<TxAttestation>, db_handle: &DBHandle) -> Result<Vec<bool>> {
    // tx_code 代表是否改变了世界状态
    let mut tx_codes: Vec<bool> = Vec::with_capacity(txs.len());

    for tx in txs {
        // 检查节点签名
        tx.self_verify().context("invalid node signature")?;
        let outcome = tx.outcome;
        // 检查用户签名
        let envelope = outcome.envelope;
        envelope.self_verify().context("invalid user signature")?;

        let receipt_opt = outcome.receipt_opt;
        let intent = envelope.intent;
        let payload = intent.payload;

        let tx_code;
        match payload {
            TxPayload::Exec { ctr_addr_str, input, .. } => {
                let ctr_addr = ContractAddress::parse_bech32m_with_id(intent.chain_id, &ctr_addr_str)?;
                let ctr_addr_bytes = ctr_addr.to_bytes();
                let ctr = db_handle.load_contract(&ctr_addr_bytes)?
                    .ok_or_else(|| anyhow!("invalid contract address"))?;
                let image_id = ctr.image_id;
                let receipt = receipt_opt.ok_or_else(|| anyhow!("exec tx must have receipt"))?;
                receipt.verify(image_id).context("verify receipt failed")?;

                let ctr_output_bytes: Vec<u8> = receipt.journal.decode().context("receipt decode failed")?;
                let ctr_output = CtrOutput::try_decode_bcs(&ctr_output_bytes).context("output decode failed")?;
                tracing::info!(target:"node::event", input=?input, output=?ctr_output);
                let ctx_hash = ctr_output.ctx_hash;
                let read_set = ctr_output.read_set;
                let ctr_ctx = CtrContext {
                    chain_id: intent.chain_id,
                    input,
                    read_set: read_set.clone()
                };
                
                ensure!(ctx_hash == sha256(ctr_ctx.encode_bcs()), "input hash mismatched");

                match &ctr_output.ctr_result {
                    CtrResult::Ok { outcome } => {
                        tx_code = db_handle.apply_rw_set(&read_set, &outcome.write_set)?;
                    },
                    CtrResult::Err { message } => {
                        tx_code = false;
                        tracing::error!(target: "node::event", %message, "ctr exec failed");
                    }
                };
            },
            TxPayload::Deploy { image_id, elf_hash, elf } => {
                let ctr = Contract::create(intent.chain_id, &elf_hash, &image_id,
                                           &envelope.verifying_key, intent.nonce);
                let ctr_addr_bytes = ctr.addr.to_bytes();
                // key: 合约的 addr 字节数组
                // value: 合约的BCS编码
                db_handle.save_contract(&ctr_addr_bytes, &ctr.to_canonical_bytes())?;
                // key: ELF文件哈希
                // value: ELF文件字节数组
                db_handle.save_elf(ctr.elf_hash, &elf)?;
                tx_code = true;
                tracing::info!(target:"node::event", contract_addr=?ctr.addr, "contract deployed");
            }
        }
        tx_codes.push(tx_code);
    }
    tracing::info!(target: "node::event", ?tx_codes);
    Ok(tx_codes)
}