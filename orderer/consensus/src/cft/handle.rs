use libp2p::PeerId;
use tokio::sync::mpsc::UnboundedSender;
use crate::solo::handle::SoloCmd;

pub enum CftCmd {
    NewSlot,
    AppendAck { peer_id: PeerId, is_ack: bool },
    ProposeBlock { block_bytes: Vec<u8> },
    CommitBlock {},
    SubmitTx { tx_bytes: Vec<u8> },
}

pub enum CftEvent {
    BlockCommited { block_bytes: Vec<u8> },
}

pub struct CftCmdHandle {
    sender: UnboundedSender<CftCmd>
}

impl CftCmdHandle {
    pub fn new(sender: UnboundedSender<CftCmd>) -> Self {
        Self { sender }
    }
}

pub struct CftEventHandle {
    sender: UnboundedSender<CftEvent>
}

impl CftEventHandle {
    pub fn new(sender: UnboundedSender<CftEvent>) -> Self {
        Self { sender }
    }
}
