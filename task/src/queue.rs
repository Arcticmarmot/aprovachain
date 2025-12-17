use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use tx::intent::TxScale;
use crate::strategy::{BySmallestTask, Strategy, Task};

#[derive(Debug, Clone)]
pub struct TaskQueue {
    pub strategy: Arc<Mutex<Strategy>>,
    pub notify: Arc<Notify>
}

impl TaskQueue {
    pub fn new_fifo() -> Self {
        Self {
            strategy: Arc::new(Mutex::new(Strategy::Fifo(VecDeque::new()))),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn new_smallest_first() -> Self {
        Self {
            strategy: Arc::new(Mutex::new(Strategy::SmallestFirst(BinaryHeap::new()))),
            notify: Arc::new(Notify::new()),
        }
    }

    pub async fn push(&self, bytes: &[u8], tx_scale: TxScale) {
        let task = Task {
            bytes: Vec::from(bytes),
            scale: tx_scale.to_slot_count()
        };
        let mut strategy_guard = self.strategy.lock().await;
        tracing::info!(target: "task::queue", ?task, "push task");
        match &mut *strategy_guard {
            Strategy::Fifo(queue) => {
                queue.push_back(task);
            },
            Strategy::SmallestFirst(queue) => {
                queue.push(Reverse(BySmallestTask(task)));
            }
        }
        tracing::info!(target: "task::queue", len=%strategy_guard.len(), "queue len");
        drop(strategy_guard); // 防止唤醒-阻塞”抖动
        self.notify.notify_one();
    }

    pub async fn wait(&self) {
        self.notify.notified().await;
    }

    pub async fn pop_or_wait(&self) -> Vec<u8> {
        loop {
            let mut strategy_guard = self.strategy.lock().await;
            let bytes_opt = match &mut *strategy_guard {
                Strategy::Fifo(queue) => {
                    queue.pop_front().map(|t| {
                        tracing::info!(target: "task::queue", ?t, "pop task");
                        t.bytes
                    })
                }
                Strategy::SmallestFirst(queue) => {
                    queue.pop().map(|t| {
                        tracing::info!(target: "task::queue", ?t, "pop task");
                        t.0.0.bytes
                    })
                }
            };
            if let Some(bytes) = bytes_opt {
                return bytes;
            }
            drop(strategy_guard);
            self.wait().await;
        }
    }
}








