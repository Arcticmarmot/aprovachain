use std::sync::Arc;
use tokio::task::spawn_blocking;
use account::keypair::AccountSigningKey;
use db::handle::DBHandle;
use engine::execute::{build_tx_outcome, verify_and_build_envelope};
use network::handle::P2pCmdHandle;
use tx::attestation::TxAttestation;
use crate::queue::TaskQueue;
use crate::error::Result;
use tokio::sync::Semaphore;

const MAX_PROVE: usize = 2;

pub async fn run_task(db_handle: &DBHandle, cmd_handle: P2pCmdHandle,
                      sk: AccountSigningKey, queue: TaskQueue) {
    let prove_sem = Arc::new(Semaphore::new(MAX_PROVE));
    loop {
        if let Some(bytes) = queue.pop().await {
            let permit = prove_sem.clone().acquire_owned().await.unwrap();

            let db_handle = db_handle.clone();
            let cmd_handle = cmd_handle.clone();
            let sk = sk.clone();
            let _ = spawn_blocking(move || {
                let _permit = permit; // 重要：持有 permit 直到证明完成
                let _ = handle_envelope(&db_handle, &cmd_handle, &sk, bytes);
            });
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
    tracing::info!(target: "task::event", len=?tx_bytes.len(), "tx_size");

    // 广播交易
    cmd_handle.publish_tx(tx_bytes)?;
    Ok(())
}