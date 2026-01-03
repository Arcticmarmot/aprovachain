use libp2p::PeerId;
use tokio::sync::mpsc::{UnboundedSender};
use crate::error::Result;

pub enum P2pCmd {
    PublishEnvelope(Vec<u8>),
    PublishTx(Vec<u8>),
    PublishBlock(Vec<u8>),
    PublishAgreement(Vec<u8>),
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

    pub fn publish_envelope(&self, envelope_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(P2pCmd::PublishEnvelope(envelope_bytes))?;
        Ok(())
    }

    pub fn publish_block(&self, block_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(P2pCmd::PublishBlock(block_bytes))?;
        Ok(())
    }

    pub fn publish_agreement(&self, agreement_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(P2pCmd::PublishAgreement(agreement_bytes))?;
        Ok(())
    }
}

pub enum P2pEvent {
    EnvelopeReceived(Vec<u8>),
    TxReceived(Vec<u8>),
    BlockReceived(Vec<u8>),
    AgreementReceived {
        from: PeerId,
        bytes: Vec<u8>,
    },
}

#[derive(Debug, Clone)]
pub struct P2pEventHandle {
    sender: UnboundedSender<P2pEvent>
}

impl P2pEventHandle {
    pub fn new(sender: UnboundedSender<P2pEvent>) -> Self {
        Self { sender }
    }
    
    pub fn received_envelope(&self, envelope_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(P2pEvent::EnvelopeReceived(envelope_bytes))?;
        Ok(())
    }

    pub fn received_tx(&self, tx_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(P2pEvent::TxReceived(tx_bytes))?;
        Ok(())
    }

    pub fn received_block(&self, block_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(P2pEvent::BlockReceived(block_bytes))?;
        Ok(())
    }

    pub fn received_agreement(&self, from: PeerId, agreement_bytes: Vec<u8>) -> Result<()> {
        self.sender.send(P2pEvent::AgreementReceived {
            from,
            bytes: agreement_bytes
        })?;
        Ok(())
    }
}