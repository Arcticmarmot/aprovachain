use anyhow::{ensure, Context, Result, anyhow};
use chain::block::Block;
use contract::contract::Contract;
use db::handle::DBHandle;
use tx::attestation::TxAttestation;
use tx::intent::TxPayload;

pub fn handle_tx_received(tx_bytes: Vec<u8>) -> Result<()> {
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

    // 业务层校验交易
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
            TxPayload::Exec { ctr_addr_bytes, input } => {
                let ctr = db_handle.load_contract(&ctr_addr_bytes)?
                    .ok_or_else(|| anyhow!("invalid contract address"))?;
                let image_id = ctr.image_id;
                let receipt = receipt_opt.ok_or_else(|| anyhow!("exec tx must have receipt"))?;
                receipt.verify(image_id).context("verify receipt failed")?;
                // 简单的 ELF 文件， input == output
                let output: Vec<u8> = receipt.journal.decode().context("output decode failed")?;
                println!("{:?}", input);
                tracing::info!(target:"node::event", input=?input, output=?output);
            },
            TxPayload::Deploy { image_id, elf_hash, elf } => {
                let ctr = Contract::create(intent.chain_id, elf_hash, image_id,
                                           &intent.verifying_key, intent.nonce);
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

    if let Some(block) = db_handle.load_block(header.height)? {
        tracing::info!(target: "node::block", ?block, "=======BLOCK=======\r\n");
    }
    Ok(())
}