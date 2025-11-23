use tokio::sync::mpsc::UnboundedSender;
use crate::error::Result;
use crate::solo::service::SoloService;

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

/// SoloCmd::NewSlot 处理
pub fn handle_new_slot(service: &mut SoloService, solo_event_hdl: &SoloEventHandle) {
    if !service.is_leader() { return; }
    match service.pack_block() {
        Ok(block) => {
            match service.update_chain_state(block.header) {
                Ok(()) => {
                    tracing::info!(target:"consensus::event", chain=?service.chain_state, "state");
                    match solo_event_hdl.block_commited(block.encode_bcs()) {
                        Ok(()) => {
                            service.mempool_handle.clear_pending();
                            tracing::info!(target:"consensus::event", pool=?service.mempool_handle, "state");
                        }
                        Err(err) => {
                            tracing::warn!(target:"consensus::event", %err, "output event");
                        }
                    }
                }
                Err(err) => {
                    tracing::warn!(target:"consensus::event", %err, "update chain state");
                }
            }
        },
        Err(err) => {
            tracing::warn!(target:"consensus::event", %err, "pack block");
        }
    }
}

/// SoloCmd::SubmitTx 处理
pub fn handle_submit_tx(service: &mut SoloService, tx_bytes: Vec<u8>) {
    if !service.is_leader() { return; }
    match service.mempool_handle.received_tx(tx_bytes) {
        Ok(()) => {
            tracing::info!(target:"consensus::event", "pushed tx");
        }
        Err(err) => {
            tracing::warn!(target:"consensus::event", %err, "received tx");
        }
    }
}






