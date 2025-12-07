use std::time::Duration;
use libp2p::PeerId;
use tokio::spawn;
use tokio::sync::mpsc::{UnboundedReceiver};
use tokio::time::sleep;
use chain::block::{OrderedBlock, BlockHeader};
use chain::chain::ChainState;
use chain::mempool::{MempoolHandle};
use primitives::constant::SLOT_TIME;
use crate::error::Result;
use crate::solo::handle::{handle_new_slot, handle_submit_tx, SoloCmd, SoloCmdHandle, SoloEventHandle};

pub const PACK_TX_COUNT: usize = 100;

pub struct SoloService {
    pub local_id: PeerId,
    pub leader_id: PeerId,
    pub chain_state: ChainState,
    pub mempool_handle: MempoolHandle
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

    pub fn pack_block(&mut self) -> Result<OrderedBlock> {
        match self.chain_state.tip_header_opt {
            Some(tip_header) => {
                let block = self.mempool_handle.pack_block(&tip_header, PACK_TX_COUNT)?;
                Ok(block)
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

pub async fn slot_loop(solo_cmd_handle: SoloCmdHandle) {
    loop {
        sleep(Duration::from_secs(SLOT_TIME)).await;
        let _ = solo_cmd_handle.new_slot();
    }
}

pub async fn start_consensus(mut service: SoloService,
                             mut solo_cmd_rx: UnboundedReceiver<SoloCmd>,
                             solo_cmd_hdl: SoloCmdHandle,
                             solo_event_hdl: SoloEventHandle) -> Result<()> {
    spawn(async move {
        slot_loop(solo_cmd_hdl).await
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