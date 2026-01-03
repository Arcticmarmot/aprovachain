use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::time::Duration;
use libp2p::PeerId;
use tokio::spawn;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::sleep;
use chain::block::{BlockHeader, OrderedBlock};
use chain::chain::ChainState;
use chain::mempool::MempoolHandle;
use primitives::constant::SLOT_SECS;
use crate::error::Result;
use platform::config::ConsensusConfig;
use primitives::hash::Hash32;
use crate::cft::handle::{handle_new_slot, handle_submit_agreement, handle_submit_tx};
use crate::cft::protocol::{CftCmd, CftCmdHandle, CftEventHandle};

pub const PACK_TX_COUNT: usize = 100;

#[derive(Debug)]
pub struct CftService {
    pub local_id: PeerId,
    pub leader_id: PeerId,
    pub chain_state: ChainState,
    pub mempool_handle: MempoolHandle,
    pub members: Vec<PeerId>,
    pub quorum: usize,
    pub pending_acks: HashMap<u128, HashSet<PeerId>>,
    pub staged_blocks: HashMap<u128, (Hash32, Vec<u8>)>,
}

impl CftService {
    pub fn new(local_id: PeerId,
               chain_state: ChainState,
               mempool_handle: MempoolHandle,
               cons_config: ConsensusConfig) -> Result<Self> {
        let leader_id = PeerId::from_str(&cons_config.leader_id).unwrap();
        let mut members = Vec::new();
        for m in cons_config.members {
            let member_id = PeerId::from_str(&m).unwrap();
            members.push(member_id);
        }
        let quorum = members.len() / 2 + 1;
        Ok(Self {
            local_id,
            leader_id,
            chain_state,
            mempool_handle,
            members,
            quorum,
            pending_acks: HashMap::new(),
            staged_blocks: HashMap::new()
        })
    }

    pub fn pack_block(&mut self) -> Result<OrderedBlock> {
        match self.chain_state.tip_header_opt {
            Some(tip_header) => {
                let block = self.mempool_handle.pack_block(&tip_header, crate::solo::service::PACK_TX_COUNT)?;
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

pub async fn slot_loop(cft_cmd_handle: CftCmdHandle) {
    loop {
        sleep(Duration::from_secs(SLOT_SECS)).await;
        let _ = cft_cmd_handle.new_slot();
    }
}



pub async fn start_cft_consensus(mut service: CftService,
                             mut cft_cmd_rx: UnboundedReceiver<CftCmd>,
                             cft_cmd_hdl: CftCmdHandle,
                             cft_event_hdl: CftEventHandle) -> Result<()> {
    let slot_cmd_hdl = cft_cmd_hdl.clone();
    spawn(async move {
        slot_loop(slot_cmd_hdl).await
    });
    let service = &mut service;
    loop {
        tokio::select! {
            Some(input) = cft_cmd_rx.recv() => {
                match input {
                    CftCmd::NewSlot => {
                        tracing::info!(target:"consensus::event", "tick tock");
                        if let Err(err) = handle_new_slot(service, &cft_event_hdl) {
                            tracing::info!(target:"consensus::event", %err);
                        }
                    },
                    CftCmd::SubmitAgreement { from, agreement_bytes } => {
                        tracing::info!(target:"consensus::event", "submit agreement");
                        if let Err(err) = handle_submit_agreement(service, &cft_event_hdl, from, agreement_bytes) {
                            tracing::info!(target:"consensus::event", %err);
                        }
                    },
                    CftCmd::SubmitTx {tx_bytes}=> {
                        tracing::info!(target:"consensus::event", "received tx");
                        if let Err(err) = handle_submit_tx(service, tx_bytes) {
                            tracing::info!(target:"consensus::event", %err);
                        }
                    },
                }
            }
        }
    }
}
