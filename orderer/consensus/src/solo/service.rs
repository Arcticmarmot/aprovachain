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

pub struct SoloService {
    pub mode: String,
    pub simulate_size: usize,
    pub simulate_size_array: Vec<usize>,
    pub simulate_index: usize,
    pub slot_secs: u64,
    pub local_id: PeerId,
    pub leader_id: PeerId,
    pub chain_state: ChainState,
    pub mempool_handle: MempoolHandle,
    pub tx_capacity: usize,
    pub is_genesis_pack: bool,
    pub is_deploy_pack: bool,
    pub is_packing: bool,
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
            simulate_size_array: cons_config.simulate_size_array,
            simulate_index: 0,
            slot_secs,
            local_id,
            leader_id,
            chain_state,
            mempool_handle,
            tx_capacity: cons_config.tx_capacity,
            is_genesis_pack: true,
            is_deploy_pack: true,
            is_packing: false,
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
                        let simulate_size;
                        if self.simulate_size != 0 {
                            simulate_size = self.simulate_size;
                        } else {
                            tracing::info!(target: "solo::event", simulate_index=%self.simulate_index);
                            if self.simulate_index >= self.simulate_size_array.len() {
                                simulate_size = 0;
                            } else {
                                simulate_size = self.simulate_size_array[self.simulate_index];
                                self.simulate_index += 1;
                            }
                        }
                        let block = self.mempool_handle
                            .simulate_pack_block(&tip_header, self.tx_capacity, simulate_size)?;

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
                             solo_event_hdl: SoloEventHandle,
                             slot_trigger: String,
) -> Result<()> {
    let slot_secs = service.slot_secs;
    let service = &mut service;
    if slot_trigger == "time" {
        let time_solo_cmd_hdl = solo_cmd_hdl.clone();
        spawn(async move {
            slot_loop(time_solo_cmd_hdl, slot_secs).await
        });
    }
    if service.is_genesis_pack {
        sleep(Duration::from_secs(slot_secs)).await;
        let _ = solo_cmd_hdl.new_slot();
    }
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
                        handle_submit_tx(service, &solo_cmd_hdl, slot_trigger.clone(), tx_bytes);
                    },
                }
            }
        }
    }
}

/// SoloCmd::NewSlot 处理
pub fn handle_new_slot(service: &mut SoloService, solo_event_hdl: &SoloEventHandle) {
    if !service.is_leader() {
        return;
    }

    // 已经有一个出块循环在跑了，新的 new_slot 只是重复唤醒，直接忽略
    if service.is_packing {
        tracing::debug!(
            target: "consensus::event",
            "pack loop is already running, ignore duplicated new slot"
        );
        return;
    }

    service.is_packing = true;

    loop {
        if service.is_genesis_pack {
            match pack_commit_broadcast_once(service, solo_event_hdl) {
                Ok(()) => {}
                Err(err) => {
                    tracing::warn!(target: "consensus::event",%err,"pack commit broadcast once failed");
                    break;
                }
            }
            service.is_genesis_pack = false;
            break;
        }

        if service.is_deploy_pack {
            match pack_commit_broadcast_once(service, solo_event_hdl) {
                Ok(()) => {}
                Err(err) => {
                    tracing::warn!(target: "consensus::event",%err,"pack commit broadcast once failed");
                    break;
                }
            }
            service.is_deploy_pack = false;
            break;
        }
        // size 模式下，如果 mempool 不够一个块，就停止
        if service.mempool_handle.mempool_count() < service.tx_capacity {
            break;
        }

        match pack_commit_broadcast_once(service, solo_event_hdl) {
            Ok(()) => {}
            Err(err) => {
                tracing::warn!(target: "consensus::event",%err,"pack commit broadcast once failed");
                break;
            }
        }
    }

    service.is_packing = false;
}


fn pack_commit_broadcast_once(
    service: &mut SoloService,
    solo_event_hdl: &SoloEventHandle,
) -> Result<()> {
    let block = service.pack_block()?;

    let height = block.header.height;
    let block_bytes = block.encode_bcs();

    tracing::info!(
        target: "consensus::event",
        height = height,
        "pack block done"
    );

    service.update_chain_state(block.header)?;

    tracing::info!(
        target: "consensus::event",
        height = height,
        chain = ?service.chain_state,
        "update chain state done"
    );

    // 关键：必须在这里同步发送，不能 spawn 一个异步任务去 broadcast
    solo_event_hdl.block_commited(block_bytes)?;

    tracing::warn!(
        target: "consensus::event",
        height = height,
        "block committed event sent"
    );

    service.mempool_handle.clear_pending();
    Ok(())
}

/// SoloCmd::SubmitTx 处理
pub fn handle_submit_tx(service: &mut SoloService, solo_cmd_handle: &SoloCmdHandle, slot_trigger: String, tx_bytes: Vec<u8>) {
    if !service.is_leader() { return; }
    match service.mempool_handle.received_tx(tx_bytes) {
        Ok(()) => {
            tracing::info!(target:"consensus::event", "pushed tx");
            if slot_trigger == "size" {
                if service.is_deploy_pack {
                    let _ = solo_cmd_handle.new_slot();
                } else {
                    if service.mempool_handle.is_ready_to_pack(service.tx_capacity){
                        let _ = solo_cmd_handle.new_slot();
                    }
                }
            }
        }
        Err(err) => {
            tracing::warn!(target:"consensus::event", %err, "received tx");
        }
    }
}

