use libp2p::PeerId;
use chain::block::OrderedBlock;
use crate::cft::agreement::Agreement;
use crate::cft::protocol::{CftEventHandle};
use crate::cft::service::CftService;
use crate::error::Result;

pub fn handle_new_slot(service: &mut CftService, cft_event_hdl: &CftEventHandle) -> Result<()> {
    if !service.is_leader() { return Ok(()); }

    if !service.pending_acks.is_empty() || !service.staged_blocks.is_empty() {
        return Ok(());
    }
    let block = service.pack_block()?;
    let header = block.header.clone();
    let height = header.height;
    let block_hash = header.hash();
    let block_bytes = block.encode_bcs();

    service.staged_blocks.insert(height, block_bytes.clone());
    service
        .pending_acks
        .entry(height)
        .or_default()
        .insert(service.local_id);
    let agr = Agreement::ProposeBlock {
        height,
        block_hash,
        block_bytes,
    };
    cft_event_hdl.agreement_commited(agr.encode_bcs())?;
    Ok(())
}

pub fn handle_submit_agreement(service: &mut CftService,
                               cft_event_hdl: &CftEventHandle,
                               from: PeerId,
                               bytes: Vec<u8>) -> Result<()> {
    let agr = Agreement::try_decode_bcs(&bytes)?;
    match agr {
        Agreement::ProposeBlock { height, block_hash, block_bytes } => {
            if service.is_leader() { return Ok(()); }

            if from != service.leader_id { return Ok(()); }

            // 解码 block + 连续性检查
            let block = OrderedBlock::try_decode_bcs(&block_bytes)?;
            let header = &block.header;

            if header.height != height || header.hash() != block_hash {
                let nack = Agreement::AppendAck { height, block_hash, ack: false };
                cft_event_hdl.agreement_commited(nack.encode_bcs())?;
                return Ok(());
            }

            if let Some(existing_bytes) = service.staged_blocks.get(&height) {
                let existing_block = OrderedBlock::try_decode_bcs(existing_bytes)?;
                if existing_block.header.hash() != block_hash {
                    let nack = Agreement::AppendAck { height, block_hash, ack: false };
                    let _ = cft_event_hdl.agreement_commited(nack.encode_bcs());
                    return Ok(());
                }
                let ack = Agreement::AppendAck { height, block_hash, ack: true };
                cft_event_hdl.agreement_commited(ack.encode_bcs())?;
                return Ok(());
            }

            match service.chain_state.tip_header_opt {
                Some(tip) => {
                    if header.height != tip.height + 1 || header.parent_hash != tip.hash() {
                        let nack = Agreement::AppendAck { height, block_hash, ack: false };
                        cft_event_hdl.agreement_commited(nack.encode_bcs())?;
                        return Ok(());
                    }
                }
                None => {
                    if header.height != 0 {
                        let nack = Agreement::AppendAck { height, block_hash, ack: false };
                        cft_event_hdl.agreement_commited(nack.encode_bcs())?;
                        return Ok(());
                    }
                }
            }

            // 暂存
            service.staged_blocks.insert(height, block_bytes);

            // 回 ack
            let ack_msg = Agreement::AppendAck { height, block_hash, ack: true };
            cft_event_hdl.agreement_commited(ack_msg.encode_bcs())?;

            Ok(())
        }
        Agreement::AppendAck { height, block_hash, ack} => {
            if !service.is_leader() { return Ok(()); }

            if !ack {
                tracing::warn!(target:"consensus::event", ?from, height, "received NACK");
                return Ok(());
            }

            // 只接受成员的 ack
            if !service.members.contains(&from) {
                return Ok(());
            }

            // 必须存在对应的 staged 提案
            let staged_bytes = match service.staged_blocks.get(&height) {
                Some(b) => b.clone(),
                None => return Ok(()),
            };

            // hash 必须匹配 staged
            let staged_block = OrderedBlock::try_decode_bcs(&staged_bytes)?;
            if staged_block.header.height != height || staged_block.header.hash() != block_hash {
                tracing::warn!(target:"consensus::event", height, "ack hash mismatch");
                return Ok(());
            }

            let acks = service.pending_acks.entry(height).or_default();
            acks.insert(from);

            if acks.len() < service.quorum {
                return Ok(());
            }
            // 达 quorum：commit 本地 tip
            service.update_chain_state(staged_block.header)?;

            // 广播 Commit（让 follower apply）
            let commit = Agreement::CommitBlock { height, block_hash };
            cft_event_hdl.agreement_commited(commit.encode_bcs())?;

            cft_event_hdl.block_commited(staged_bytes.clone())?;

            // 清理 pending + staged + mempool
            service.pending_acks.remove(&height);
            service.staged_blocks.remove(&height);
            service.mempool_handle.clear_pending();

            Ok(())
        }
        Agreement::CommitBlock { height, block_hash } => {
            if service.is_leader() { return Ok(()) }
            if from != service.leader_id { return Ok(()); }

            let staged_bytes = match service.staged_blocks.get(&height) {
                Some(b) => b.clone(),
                None => return Ok(()),
            };

            let staged_block = OrderedBlock::try_decode_bcs(&staged_bytes)?;
            if staged_block.header.height != height || staged_block.header.hash() != block_hash {
                tracing::warn!(target:"consensus::event", height, "commit hash mismatch");
                return Ok(());
            }

            // 连续性（防止乱序 commit）
            match service.chain_state.tip_header_opt {
                Some(tip) => {
                    if staged_block.header.height != tip.height + 1 {
                        return Ok(());
                    }
                    if staged_block.header.parent_hash != tip.hash() {
                        return Ok(());
                    }
                }
                None => {
                    if staged_block.header.height != 0 {
                        return Ok(());
                    }
                }
            }

            service.update_chain_state(staged_block.header.clone())?;
            service.staged_blocks.remove(&height);
            Ok(())
        }
    }
}


pub fn handle_submit_tx(service: &mut CftService, tx_bytes: Vec<u8>) -> Result<()> {
    if !service.is_leader() { return Ok(()) }
    service.mempool_handle.received_tx(tx_bytes)?;
    Ok(())
}