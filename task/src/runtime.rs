use account::executor::ExecutorId;
use account::keypair::AccountSigningKey;
use db::handle::DBHandle;
use engine::execute::{build_tx_outcome, verify_and_build_envelope};
use network::handle::P2pCmdHandle;
use schedule::dispatch::assign_executor_for_tx;
use tx::attestation::TxAttestation;
use tx::envelope::TxEnvelope;
use crate::queue::TaskQueue;
use crate::error::Result;

pub async fn run_task(db_handle: DBHandle, cmd_handle: P2pCmdHandle,
                      sk: AccountSigningKey, queue: TaskQueue) {
    loop {
        if let Some(bytes) = queue.pop().await {
            let _ = handle_envelope(&db_handle, &cmd_handle, &sk, bytes);
        }
        queue.wait().await;
    }
}

pub fn handle_envelope(db_handle: &DBHandle, cmd_handle: &P2pCmdHandle, sk: &AccountSigningKey,
                       bytes: Vec<u8>) -> Result<()> {
    let envelope = verify_and_build_envelope(&bytes)?;

    let outcome = build_tx_outcome(&db_handle, envelope)?;

    let tx = TxAttestation::create(outcome, sk.clone());
    let tx_bytes = tx.to_canonical_bytes();
    tracing::info!(target: "executor::event", len=?tx_bytes.len(), "tx_size");

    // 广播交易
    cmd_handle.publish_tx(tx_bytes)?;
    Ok(())
}