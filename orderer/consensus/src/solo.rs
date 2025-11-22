use std::time::Duration;
use anyhow::Result;
use libp2p::PeerId;
use tokio::spawn;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::sleep;
use chain::block::{Block, BlockHeader};
use chain::chain::ChainState;
use chain::mempool::{MempoolHandle};

pub enum InputEvent {
    Tick,
    ReceivedTx,
    ReceivedBlock
}

pub enum OutputEvent {
    CommitBlock(BlockHeader),
    ProposeBlock(Block)
}

pub const TICK_INTERVAL: Duration = Duration::from_secs(5);
pub struct SoloService {
    self_id: PeerId,
    leader_id: PeerId,
    chain_state: ChainState,
    tick_interval: Duration,
    mempool_handle: MempoolHandle
}

impl SoloService {
    pub fn new(self_id: PeerId, 
               leader_id: PeerId, 
               chain_state: ChainState, 
               tick_interval: Duration,
               mempool_handle: MempoolHandle) -> Self {
        Self {
            self_id,
            leader_id,
            chain_state,
            tick_interval,
            mempool_handle
        }
    }
    
    pub fn get_tip_block(&self) -> BlockHeader {
        self.chain_state.tip_header
    }
    
    pub fn pack_block(&self) -> Block {
        let block = self.mempool_handle.pack_block(&self.get_tip_block(), 10).unwrap();
        block
    }
}

pub async fn slap_loop(input_tx: UnboundedSender<InputEvent>) {
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
       slap_loop(input_tx).await
    });
    loop {
        tokio::select! {
            Some(input) = input_rx.recv() => {
                match input {
                    InputEvent::Tick => {
                        let block = service.pack_block();
                        output_tx.send(OutputEvent::ProposeBlock(block))?;
                    },
                    InputEvent::ReceivedTx => {
                        
                    },
                    InputEvent::ReceivedBlock => {
                        
                    }
                }
            }
        }
    }
}