use tokio::sync::mpsc::{UnboundedSender};
use crate::cmd::NetworkCmd;

#[derive(Debug, Clone)]
pub struct P2pHandle {
    tx: UnboundedSender<NetworkCmd>
}

impl P2pHandle {
    pub fn new(tx: UnboundedSender<NetworkCmd>) -> Self {
        Self { tx }
    }

    pub fn publish_tx(&self, tx_bytes: Vec<u8>) {
        let _ = self.tx.send(NetworkCmd::PublishTx(tx_bytes));
    }
}