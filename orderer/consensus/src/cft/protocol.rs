use libp2p::PeerId;
use tokio::sync::mpsc::UnboundedSender;
use crate::error::Result;

pub enum CftCmd {
    NewSlot,
    SubmitAgreement { from: PeerId, agreement_bytes: Vec<u8> },
    SubmitTx { tx_bytes: Vec<u8> },
}

pub enum CftEvent {
    AgreementCommited { agreement_bytes: Vec<u8> },
    BlockCommited { block_bytes: Vec<u8> },
}

#[derive(Debug, Clone)]
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

    pub fn submit_agreement(&self, from: PeerId, agreement_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(CftCmd::SubmitAgreement { from, agreement_bytes })?;
        Ok(())
    }

    pub fn submit_tx(&self, tx_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(CftCmd::SubmitTx {tx_bytes})?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct CftEventHandle {
    pub sender: UnboundedSender<CftEvent>
}

impl CftEventHandle {
    pub fn new(sender: UnboundedSender<CftEvent>) -> Self {
        Self { sender }
    }

    pub fn agreement_commited(&self, agreement_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(CftEvent::AgreementCommited { agreement_bytes })?;
        Ok(())
    }

    pub fn block_commited(&self, block_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(CftEvent::BlockCommited {block_bytes})?;
        Ok(())
    }
}