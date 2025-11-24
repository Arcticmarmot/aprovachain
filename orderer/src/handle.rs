use consensus::solo::handle::SoloCmdHandle;
use network::handle::P2pCmdHandle;
use tx::tx_exec_seal::{TxExecSeal, TxExecSealWire};
use anyhow::Result;

/// Orderer P2pEvent::TxReceived 处理
pub fn handle_tx_received(tx_bytes: Vec<u8>, solo_cmd_hdl: &SoloCmdHandle) -> Result<()> {
    // 验证字节数组是否是有效交易
    let wire = TxExecSealWire::try_decode_bcs(&tx_bytes)?;
    TxExecSeal::try_from(wire)?;
    solo_cmd_hdl.submit_tx(tx_bytes)?;
    Ok(())
}

/// Orderer P2pEvent::BlockReceived 处理
pub fn handle_block_received(block_bytes: Vec<u8>) -> Result<()> {
    Ok(())
}

/// Orderer SoloEvent::BlockCommited 处理
pub fn handle_block_commited(block_bytes: Vec<u8>, p2p_cmd_hdl: &P2pCmdHandle) -> Result<()> {
    p2p_cmd_hdl.publish_block(block_bytes)?;
    Ok(())
}