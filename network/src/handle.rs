use tokio::sync::mpsc::{UnboundedSender};
use crate::error::Result;
pub enum P2pCmd {
    PublishTx(Vec<u8>),
    PublishBlock(Vec<u8>)
}

#[derive(Debug, Clone)]
pub struct P2pHandle {
    sender: UnboundedSender<P2pCmd>
}

impl P2pHandle {
    pub fn new(sender: UnboundedSender<P2pCmd>) -> Self {
        Self { sender }
    }

    pub fn publish_tx(&self, tx_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(P2pCmd::PublishTx(tx_bytes))?;
        Ok(())
    }

    pub fn publish_block(&self, block_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(P2pCmd::PublishBlock(block_bytes))?;
        Ok(())
    }
}