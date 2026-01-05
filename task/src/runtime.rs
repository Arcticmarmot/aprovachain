use std::sync::Arc;
use tokio::sync::watch::Receiver;
use tokio::task::spawn_blocking;
use account::keypair::AccountSigningKey;
use db::handle::DBHandle;
use engine::execute::{build_tx_outcome};
use network::handle::P2pCmdHandle;
use tx::attestation::TxAttestation;
use crate::schedule::TaskSchedule;
use crate::error::Result;
use tokio::sync::Semaphore;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use tx::envelope::TxEnvelope;
use platform::config::{ ProveMode, QueueConfig};
const MAX_PROVE: usize = 1;
static INFLIGHT: AtomicUsize = AtomicUsize::new(0);

pub async fn run_task(db_handle: DBHandle, cmd_handle: P2pCmdHandle,
                      sk: AccountSigningKey, schedule: TaskSchedule, mut shutdown_rx: Receiver<bool>,
                      prove_mode: ProveMode, queue_config: QueueConfig) {
    let prove_sem = Arc::new(Semaphore::new(MAX_PROVE));
    loop {
        tokio::select! {
            bytes = schedule.pop_or_wait() => {
                let permit = prove_sem.clone().acquire_owned().await.expect("prove_sem closed");
                let db_handle = db_handle.clone();
                let cmd_handle = cmd_handle.clone();
                let sk = sk.clone();
                let prove_mode = prove_mode.clone();
                spawn_blocking(move || {
                    // NOTE: 不能使用 _ 会被直接释放丢弃
                    let _permit = permit;
                    let n = INFLIGHT.fetch_add(1, SeqCst) + 1;
                    tracing::info!(target="task::runtime", inflight=n, "prove start");
                    let result = handle_envelope(&db_handle, &cmd_handle, &sk, bytes, prove_mode);
                    let n = INFLIGHT.fetch_sub(1, SeqCst) - 1;
                    tracing::info!(target="task::runtime", inflight=n, ?result, "prove end");
                });
            },
            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    tracing::info!(target:"task::runtime", "shutdown received, stopping task loop");
                    break;
                }
            }
        }
    }
}

pub fn handle_envelope(db_handle: &DBHandle, cmd_handle: &P2pCmdHandle, sk: &AccountSigningKey,
                       envelope: TxEnvelope, prove_mode: ProveMode) -> Result<()> {
    let outcome = build_tx_outcome(&db_handle, envelope, prove_mode)?;

    let tx = TxAttestation::create(outcome, sk.clone());
    let tx_bytes = tx.to_canonical_bytes();
    tracing::info!(target: "task::runtime", len=?tx_bytes.len(), "tx_size");

    // 广播交易
    cmd_handle.publish_tx(tx_bytes.clone())?;
    Ok(())
}