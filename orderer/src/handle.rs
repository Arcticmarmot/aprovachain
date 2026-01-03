use consensus::solo::protocol::SoloCmdHandle;
use network::handle::P2pCmdHandle;
use anyhow::Result;
use engine::execute::verify_and_build_tx;

/// Orderer P2pEvent::TxReceived 处理
pub fn on_tx_received(tx_bytes: Vec<u8>, solo_cmd_hdl: &SoloCmdHandle) -> Result<()> {
    // 验证字节数组是否是有效交易
    verify_and_build_tx(&tx_bytes)?;
    solo_cmd_hdl.submit_tx(tx_bytes)?;
    Ok(())
}

/// Orderer P2pEvent::BlockReceived 处理
pub fn on_block_received(_: Vec<u8>) -> Result<()> {
    Ok(())
}

/// Orderer SoloEvent::BlockCommited 处理
pub fn on_block_proposed(block_bytes: Vec<u8>, p2p_cmd_hdl: &P2pCmdHandle) -> Result<()> {
    p2p_cmd_hdl.publish_block(block_bytes)?;
    Ok(())
}

/// Orderer SoloEvent::BlockCommited 处理
pub fn on_block_commited(block_bytes: Vec<u8>, p2p_cmd_hdl: &P2pCmdHandle) -> Result<()> {
    p2p_cmd_hdl.publish_block(block_bytes)?;
    Ok(())
}