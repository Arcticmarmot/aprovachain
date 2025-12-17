use std::collections::{BinaryHeap, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use tx::intent::TxScale;
use crate::discipline::*;
use std::sync::atomic::{AtomicU64, Ordering as AOrd};

pub enum DisciplineKind {
    Fcfs,
    Spt,
    Edf,
    SptEdf,
    EdfSpt,
}

#[derive(Debug, Clone)]
pub struct TaskSchedule {
    pub discipline: Arc<Mutex<Discipline>>,
    pub notify: Arc<Notify>,
    pub seq: Arc<AtomicU64>,
}

impl TaskSchedule {
    pub fn create(kind: DisciplineKind) -> Self {
        let discipline = match kind {
            DisciplineKind::Fcfs => Discipline::Fcfs(VecDeque::new()),
            DisciplineKind::Spt => Discipline::Spt(BinaryHeap::new()),
            DisciplineKind::Edf => Discipline::Edf(BinaryHeap::new()),
            DisciplineKind::SptEdf => Discipline::SptEdf(BinaryHeap::new()),
            DisciplineKind::EdfSpt => Discipline::EdfSpt(BinaryHeap::new()),
        };
        Self {
            discipline: Arc::new(Mutex::new(discipline)),
            notify: Arc::new(Notify::new()),
            seq: Arc::new(AtomicU64::new(0))
        }
    }

    pub async fn push(&self, bytes: &[u8], tx_scale: TxScale, send_height: u128) {
        let seq = self.seq.fetch_add(1, AOrd::Relaxed);
        let deadline = send_height + tx_scale.to_slot_count() as u128;
        let task = Task {
            bytes: Vec::from(bytes),
            scale: tx_scale.to_slot_count(),
            deadline,
            seq
        };
        let mut discipline_guard = self.discipline.lock().await;
        tracing::info!(target: "task::queue", ?task, "push task");
        match &mut *discipline_guard {
            Discipline::Fcfs(queue) => {
                queue.push_back(task);
            },
            Discipline::Spt(queue) => {
                queue.push(SptKey(task));
            },
            Discipline::Edf(queue) => {
                queue.push(EdfKey(task));
            },
            Discipline::SptEdf(queue) => {
                queue.push(SptEdfKey(task));
            },
            Discipline::EdfSpt(queue) => {
                queue.push(EdfSptKey(task));
            }
        }
        tracing::info!(target: "task::queue", len=%discipline_guard.len(), "queue len");
        drop(discipline_guard); // 防止唤醒-阻塞”抖动
        self.notify.notify_one();
    }

    pub async fn wait(&self) {
        self.notify.notified().await;
    }

    pub async fn pop_or_wait(&self) -> Vec<u8> {
        loop {
            let mut discipline_guard = self.discipline.lock().await;
            let bytes_opt = match &mut *discipline_guard {
                Discipline::Fcfs(queue) => {
                    queue.pop_front().map(|t| {
                        tracing::info!(target: "task::queue", ?t, "pop task");
                        t.bytes
                    })
                }
                Discipline::Spt(queue) => {
                    queue.pop().map(|t| {
                        tracing::info!(target: "task::queue", ?t, "pop task");
                        t.0.bytes
                    })
                }
                Discipline::Edf(queue) => {
                    queue.pop().map(|t| {
                        tracing::info!(target: "task::queue", ?t, "pop task");
                        t.0.bytes
                    })
                }
                Discipline::SptEdf(queue) => {
                    queue.pop().map(|t| {
                        tracing::info!(target: "task::queue", ?t, "pop task");
                        t.0.bytes
                    })
                }
                Discipline::EdfSpt(queue) => {
                    queue.pop().map(|t| {
                        tracing::info!(target: "task::queue", ?t, "pop task");
                        t.0.bytes
                    })
                }
            };
            if let Some(bytes) = bytes_opt {
                return bytes;
            }
            drop(discipline_guard);
            self.wait().await;
        }
    }
}








