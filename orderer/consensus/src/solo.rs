use std::time::Duration;
use libp2p::PeerId;
use tokio::spawn;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::sleep;
use chain::block::{Block, BlockHeader};
use chain::chain::ChainState;
use chain::mempool::{MempoolHandle};
use crate::error::Result;

pub enum InputEvent {
    Tick,
    ReceivedTx(Vec<u8>),
}

pub enum OutputEvent {
    CommitBlock(Vec<u8>),
}

pub const TICK_INTERVAL: Duration = Duration::from_secs(30);
pub struct SoloService {
    local_id: PeerId,
    leader_id: PeerId,
    chain_state: ChainState,
    mempool_handle: MempoolHandle
}

impl SoloService {
    pub fn new(local_id: PeerId,
               leader_id: PeerId,
               chain_state: ChainState,
               mempool_handle: MempoolHandle) -> Self {
        Self {
            local_id,
            leader_id,
            chain_state,
            mempool_handle
        }
    }

    pub fn pack_block(&mut self) -> Result<Block> {
        let tip_header = self.chain_state.tip_header;
        let block = self.mempool_handle.pack_block(&tip_header, 2)?;
        Ok(block)
    }

    pub fn update_chain_state(&mut self, header: BlockHeader) -> Result<()> {
        self.chain_state.update(header)?;
        Ok(())
    }

    pub fn is_leader(&self) -> bool {
        self.local_id == self.leader_id
    }
}

pub async fn slot_loop(input_tx: UnboundedSender<InputEvent>) {
    loop {
        sleep(TICK_INTERVAL).await;
        let _ = input_tx.send(InputEvent::Tick);
    }
}

pub async fn start_consensus(service: &mut SoloService,
                             input_tx: UnboundedSender<InputEvent>,
                             mut input_rx: UnboundedReceiver<InputEvent>,
                             output_tx: UnboundedSender<OutputEvent>) -> Result<()> {
    spawn(async move {
        slot_loop(input_tx).await
    });
    loop {
        tokio::select! {
            Some(input) = input_rx.recv() => {
                match input {
                    InputEvent::Tick => {
                        tracing::info!(target:"consensus::event", "tick tock");
                        if !service.is_leader() { continue; }
                        match service.pack_block() {
                            Ok(block) => {
                                match service.update_chain_state(block.header) {
                                    Ok(()) => {
                                        tracing::info!(target:"consensus::event", chain=?service.chain_state, "state");
                                        if let Err(err) =
                                            output_tx.send(OutputEvent::CommitBlock(block.encode_bcs())) {
                                            tracing::warn!(target:"consensus::event", %err, "output event");
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

                    },
                    InputEvent::ReceivedTx(tx_bytes)=> {
                        tracing::info!(target:"consensus::event", "received tx");
                        if !service.is_leader() { continue; }
                        match service.mempool_handle.received_tx(tx_bytes) {
                            Ok(()) => {
                                tracing::info!(target:"consensus::event", "pushed tx");
                            }
                            Err(err) => {
                                tracing::warn!(target:"consensus::event", %err, "received tx");
                            }
                        }
                    },
                }
            }
        }
    }
}