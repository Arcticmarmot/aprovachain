use tokio::sync::mpsc::{UnboundedSender};
use crate::error::Result;
use crate::handle::P2pEvent::{ReceivedBlock, ReceivedTx};

pub enum P2pCmd {
    PublishTx(Vec<u8>),
    PublishBlock(Vec<u8>)
}

#[derive(Debug, Clone)]
pub struct P2pCmdHandle {
    sender: UnboundedSender<P2pCmd>
}

impl P2pCmdHandle {
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

pub enum P2pEvent {
    ReceivedTx(Vec<u8>),
    ReceivedBlock(Vec<u8>)
}

#[derive(Debug, Clone)]
pub struct P2pEventHandle {
    sender: UnboundedSender<P2pEvent>
}

impl P2pEventHandle {
    pub fn new(sender: UnboundedSender<P2pEvent>) -> Self {
        Self { sender }
    }

    pub fn received_tx(&self, tx_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(ReceivedTx(tx_bytes))?;
        Ok(())
    }

    pub fn received_block(&self, block_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(ReceivedBlock(block_bytes))?;
        Ok(())
    }
}