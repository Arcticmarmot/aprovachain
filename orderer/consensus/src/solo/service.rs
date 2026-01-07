use std::time::Duration;
use libp2p::PeerId;
use tokio::spawn;
use tokio::sync::mpsc::{UnboundedReceiver};
use tokio::time::sleep;
use chain::block::{OrderedBlock, BlockHeader};
use chain::chain::ChainState;
use chain::mempool::{MempoolHandle};
use platform::config::ConsensusConfig;
use crate::error::{ConsensusError, Result};
use crate::solo::protocol::{SoloCmd, SoloCmdHandle, SoloEventHandle};

pub const PACK_TX_COUNT: usize = 100;

pub struct SoloService {
    pub mode: String,
    pub simulate_size: usize,
    pub slot_secs: u64,
    pub local_id: PeerId,
    pub leader_id: PeerId,
    pub chain_state: ChainState,
    pub mempool_handle: MempoolHandle,
    pub tx_capacity: usize
}

impl SoloService {
    pub fn new(slot_secs: u64,
               local_id: PeerId,
               leader_id: PeerId,
               chain_state: ChainState,
               mempool_handle: MempoolHandle,
               cons_config: ConsensusConfig) -> Self {
        Self {
            mode: cons_config.mode,
            simulate_size: cons_config.simulate_size,
            slot_secs,
            local_id,
            leader_id,
            chain_state,
            mempool_handle,
            tx_capacity: cons_config.tx_capacity
        }
    }

    pub fn pack_block(&mut self) -> Result<OrderedBlock> {
        match self.chain_state.tip_header_opt {
            Some(tip_header) => {
                match self.mode.as_str() {
                    "native" => {
                        let block = self.mempool_handle.pack_block(&tip_header, self.tx_capacity)?;
                        Ok(block)
                    }
                    "simulate" => {
                        let block = self.mempool_handle
                            .simulate_pack_block(&tip_header, self.tx_capacity, self.simulate_size)?;
                        Ok(block)
                    }
                    _ => {
                        Err(ConsensusError::ConsensusMode)
                    }
                }
            },
            None => {
                let genesis = OrderedBlock::genesis()?;
                Ok(genesis)
            }
        }
    }

    pub fn update_chain_state(&mut self, header: BlockHeader) -> Result<()> {
        self.chain_state.update(header)?;
        Ok(())
    }

    pub fn is_leader(&self) -> bool {
        self.local_id == self.leader_id
    }
}

pub async fn slot_loop(solo_cmd_handle: SoloCmdHandle, slot_secs: u64) {
    loop {
        sleep(Duration::from_secs(slot_secs)).await;
        let _ = solo_cmd_handle.new_slot();
    }
}

pub async fn start_solo_consensus(mut service: SoloService,
                             mut solo_cmd_rx: UnboundedReceiver<SoloCmd>,
                             solo_cmd_hdl: SoloCmdHandle,
                             solo_event_hdl: SoloEventHandle) -> Result<()> {
    let slot_secs = service.slot_secs;
    spawn(async move {
        slot_loop(solo_cmd_hdl, slot_secs).await
    });
    let service = &mut service;
    loop {
        tokio::select! {
            Some(input) = solo_cmd_rx.recv() => {
                match input {
                    SoloCmd::NewSlot => {
                        tracing::info!(target:"consensus::event", "tick tock");
                        handle_new_slot(service, &solo_event_hdl);
                    },
                    SoloCmd::SubmitTx {tx_bytes}=> {
                        tracing::info!(target:"consensus::event", "received tx");
                        handle_submit_tx(service, tx_bytes);
                    },
                }
            }
        }
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
