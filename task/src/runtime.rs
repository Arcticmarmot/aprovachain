use std::sync::Arc;
use tokio::sync::watch::Receiver;
use tokio::task::spawn_blocking;
use account::keypair::AccountSigningKey;
use db::handle::DBHandle;
use engine::execute::{build_tx_outcome, verify_and_build_envelope};
use network::handle::P2pCmdHandle;
use tx::attestation::TxAttestation;
use crate::queue::TaskQueue;
use crate::error::Result;
use tokio::sync::Semaphore;

const MAX_PROVE: usize = 1;

pub async fn run_task(db_handle: DBHandle, cmd_handle: P2pCmdHandle,
                      sk: AccountSigningKey, queue: TaskQueue, mut shutdown_rx: Receiver<bool>) {
    let prove_sem = Arc::new(Semaphore::new(MAX_PROVE));
    loop {
        tokio::select! {
            bytes = queue.pop_or_wait() => {
                let permit = prove_sem.clone().acquire_owned().await.expect("prove_sem closed");
                let db_handle = db_handle.clone();
                let cmd_handle = cmd_handle.clone();
                let sk = sk.clone();
                let _ = spawn_blocking(move || {
                    // NOTE: 不能使用 _ 会被直接释放丢弃
                    let _permit = permit;
                    let _ = handle_envelope(&db_handle, &cmd_handle, &sk, bytes);
                });
            },
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    tracing::info!(target:"net::signal", "shutdown received, stopping task loop");
                    break;
                }
            }
        }
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