use anyhow::{ensure, Context, Result, anyhow};
use chain::block::Block;
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
        None => { }
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

        let payload = &envelope.intent.payload;
        let receipt = &tx.outcome.receipt;
        match payload {
            TxPayload::Exec { ctr_addr, input } => {
                // TODO: 合约需要部署在所有节点上
                let ctr = db_handle.load_contract(&ctr_addr)?
                    .ok_or_else(|| anyhow!("invalid contract address"))?;
                let image_id = ctr.image_id;
                receipt.verify(image_id).context("verify receipt failed")?;
                // 简单的 ELF 文件， input == output
                let output: Vec<u8> = receipt.journal.decode().context("output decode failed")?;
                println!("{:?}", input);
                tracing::info!(target:"node::event", input=?input, output=?output);
            },
            TxPayload::Deploy { .. } => { }
        }
    }

    // 存储 block
    db_handle.save_block(&block)?;

    // 更新 chain_state
    db_handle.save_chain_state(&header)?;

    if let Some(block) = db_handle.load_block(header.height)? {
        tracing::info!(target: "==BLOCK==", ?block);
    }
    Ok(())
}