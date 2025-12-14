use anyhow::{Result};
use account::executor::ExecutorId;
use account::keypair::AccountSigningKey;
use db::handle::DBHandle;
use engine::execute::{build_tx_outcome, verify_and_build_envelope};
use engine::verify::verify_and_apply_block;
use network::handle::P2pCmdHandle;
use schedule::dispatch::assign_executor_for_tx;
use tx::attestation::TxAttestation;

pub fn on_envelope_received(cmd_handle: P2pCmdHandle, sk: AccountSigningKey, envelope_bytes: Vec<u8>) -> Result<()> {
    // 加载状态信息
    let db_handle = DBHandle::new()?;
    let self_exec_id = ExecutorId(sk.verifying_key());

    // 从字节数组构造 TxEnvelope
    let envelope = verify_and_build_envelope(&envelope_bytes)?;
    let tx_envelope_id = envelope.tx_id();

    let exec_id = match assign_executor_for_tx(&db_handle, &tx_envelope_id, envelope.intent.timestamp)? {
        Some(exec_id) => { exec_id },
        None => {
            tracing::info!(target: "executor::event", %tx_envelope_id, "no metrics yet, fall back to self as executor");
            self_exec_id
        }
    };
    tracing::info!(target:"executor::event", %exec_id, "executor id");

    if exec_id == self_exec_id {
        tracing::info!(target:"executor::event", "I will do it");
        // 执行交易
        let outcome = build_tx_outcome(&db_handle, envelope)?;

        let tx = TxAttestation::create(outcome, sk);
        let tx_bytes = tx.to_canonical_bytes();
        tracing::info!(target: "executor::event", len=?tx_bytes.len(), "tx_size");

        // 广播交易
        cmd_handle.publish_tx(tx_bytes)?;
    } else {
        tracing::info!(target:"executor::event", "none of my business");
    }
    Ok(())
}

pub fn on_block_received(db_handle: &DBHandle, block_bytes: Vec<u8>) -> Result<()> {
    verify_and_apply_block(db_handle, block_bytes)
}