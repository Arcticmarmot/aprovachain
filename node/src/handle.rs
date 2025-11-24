use anyhow::{Context, Error, Result};
use account::address::ContractAddress;
use chain::block::Block;
use chain::error::ChainError::InvalidBlock;
use db::handle::DBHandle;
use tx::tx_exec_seal::TxExecSeal;
use tx::tx_intent::TxPayload;

pub fn handle_tx_received(tx_bytes: Vec<u8>) -> Result<()> {
    Ok(())
}

pub fn handle_block_received(block_bytes: Vec<u8>) -> Result<()> {
    let block = Block::try_decode_bcs(&block_bytes)?;
    let header = block.header;
    let db_handle = DBHandle::new()?;
    // 验证是否是合法区块，并存储 chain_state
    let chain_state = db_handle.load_chain_state()?;
    match chain_state {
        Some(tip_header) => {
            if header.height != tip_header.height + 1 { return Err(Error::new(InvalidBlock)); }
            if header.parent_hash != tip_header.hash() { return Err(Error::new(InvalidBlock)); }
        },
        None => { }
    }

    // 验证区块内交易
    let txs: Vec<TxExecSeal> = block.txs.iter()
        .map(|tx| TxExecSeal::try_from(tx.clone()).unwrap() ).collect();
    for tx in txs {
        let payload = tx.exec.envelope.intent.payload;
        let receipt = tx.exec.receipt;
        match payload {
            TxPayload::Exec { ctr_addr, input } => {
                match db_handle.load_contract(&ctr_addr)? {
                    Some(ctr) => {
                        let image_id = ctr.image_id;
                        receipt.verify(image_id).context("verify receipt failed")?;
                        // 简单的 ELF 文件， input == output
                        let output: Vec<u8> = receipt.journal.decode().context("output decode failed")?;
                        if output != input {
                            return Err(Error::new(InvalidBlock));
                        }
                    },
                    None => {
                        return Err(Error::new(InvalidBlock));
                    }
                };
            },
            TxPayload::Deploy { .. } => { }
        }
    }

    // 存储 block
    db_handle.save_block(&block)?;

    // 更新 chain_state
    db_handle.save_chain_state(&header)?;
    Ok(())
}