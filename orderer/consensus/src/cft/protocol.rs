use libp2p::PeerId;
use tokio::sync::mpsc::UnboundedSender;
use crate::error::Result;

pub enum CftCmd {
    NewSlot,
    AppendAck { peer_id: PeerId, ack: bool },
    ProposeBlock { block_bytes: Vec<u8> },
    CommitBlock { block_bytes: Vec<u8> },
    SubmitTx { tx_bytes: Vec<u8> },
}

pub enum CftEvent {
    BlockProposed { block_bytes: Vec<u8> },
    BlockCommited { block_bytes: Vec<u8> },
}

pub struct CftCmdHandle {
    pub sender: UnboundedSender<CftCmd>
}

impl CftCmdHandle {
    pub fn new(sender: UnboundedSender<CftCmd>) -> Self {
        Self { sender }
    }
    
    pub fn new_slot(&self) -> Result<()> {
        self.sender.send(CftCmd::NewSlot)?;
        Ok(())
    }
    
    pub fn append_ack(&self) -> Result<()> {
        self.sender.send(CftCmd::NewSlot)?;
        Ok(())
    }
    
    pub fn propose_block(&self, block_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(CftCmd::ProposeBlock { block_bytes })?;
        Ok(())
    }

    pub fn commit_block(&self, block_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(CftCmd::CommitBlock { block_bytes })?;
        Ok(())
    }

    pub fn submit_tx(&self, tx_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(CftCmd::SubmitTx {tx_bytes})?;
        Ok(())
    }
}

pub struct CftEventHandle {
    pub sender: UnboundedSender<CftEvent>
}

impl CftEventHandle {
    pub fn new(sender: UnboundedSender<CftEvent>) -> Self {
        Self { sender }
    }
    
    pub fn block_proposed(&self, block_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(CftEvent::BlockProposed {block_bytes})?;
        Ok(())
    }

    pub fn block_commited(&self, block_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(CftEvent::BlockCommited {block_bytes})?;
        Ok(())
    }
}
