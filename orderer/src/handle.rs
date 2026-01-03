use consensus::solo::protocol::SoloCmdHandle;
use network::handle::P2pCmdHandle;
use anyhow::Result;
use libp2p::PeerId;
use consensus::cft::protocol::{CftCmdHandle};
use engine::execute::verify_and_build_tx;

/// Orderer Solo P2pEvent::TxReceived 处理
pub fn on_solo_tx_received(tx_bytes: Vec<u8>, solo_cmd_hdl: &SoloCmdHandle) -> Result<()> {
    // 验证字节数组是否是有效交易
    verify_and_build_tx(&tx_bytes)?;
    solo_cmd_hdl.submit_tx(tx_bytes)?;
    Ok(())
}
/// Orderer Cft P2pEvent::TxReceived 处理
pub fn on_cft_tx_received(tx_bytes: Vec<u8>, cft_cmd_hdl: &CftCmdHandle) -> Result<()> {
    // 验证字节数组是否是有效交易
    verify_and_build_tx(&tx_bytes)?;
    cft_cmd_hdl.submit_tx(tx_bytes)?;
    Ok(())
}

/// Orderer Cft P2pEvent::AgreementReceived 处理
pub fn on_cft_agreement_received(from: PeerId, agreement_bytes: Vec<u8>, cft_cmd_hdl: &CftCmdHandle) -> Result<()> {
    cft_cmd_hdl.submit_agreement(from, agreement_bytes)?;
    Ok(())
}
pub fn on_cft_agreement_commited(agreement_bytes: Vec<u8>, p2p_cmd_hdl: &P2pCmdHandle) -> Result<()> {
    p2p_cmd_hdl.publish_agreement(agreement_bytes)?;
    Ok(())
}
/// Orderer SoloEvent::BlockCommited 处理
pub fn on_block_commited(block_bytes: Vec<u8>, p2p_cmd_hdl: &P2pCmdHandle) -> Result<()> {
    p2p_cmd_hdl.publish_block(block_bytes)?;
    Ok(())
}