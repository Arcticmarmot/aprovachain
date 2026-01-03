use tokio::sync::mpsc::UnboundedSender;
use crate::error::Result;

pub enum SoloCmd {
    NewSlot,
    SubmitTx { tx_bytes: Vec<u8> },
}

pub enum SoloEvent {
    BlockCommited { block_bytes: Vec<u8> },
}

#[derive(Debug, Clone)]
pub struct SoloCmdHandle {
    sender: UnboundedSender<SoloCmd>
}

impl SoloCmdHandle {
    pub fn new(sender: UnboundedSender<SoloCmd>) -> Self {
        Self { sender }
    }
    
    pub fn new_slot(&self) -> Result<()> {
        self.sender.send(SoloCmd::NewSlot)?;
        Ok(())
    }
    
    pub fn submit_tx(&self, tx_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(SoloCmd::SubmitTx {tx_bytes})?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SoloEventHandle {
    sender: UnboundedSender<SoloEvent>
}

impl SoloEventHandle {
    pub fn new(sender: UnboundedSender<SoloEvent>) -> Self {
        Self { sender }
    }
    
    pub fn block_commited(&self, block_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(SoloEvent::BlockCommited {block_bytes})?;
        Ok(())
    }
}





