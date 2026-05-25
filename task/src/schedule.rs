use std::collections::{BinaryHeap, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use crate::discipline::*;
use std::sync::atomic::{AtomicU64, Ordering as AOrd};
use tx::envelope::TxEnvelope;

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

    pub async fn push(&self, envelope: TxEnvelope, scale: u32, send_height: u128) {
        let seq = self.seq.fetch_add(1, AOrd::Relaxed);
        let tx_scale = envelope.intent.scale; // 用户预估 tx_scale 用于计算截止时间
        let deadline = send_height + tx_scale.to_slot_count() as u128;
        let scale = scale;
        let task = Task { envelope, scale, deadline, seq };
        let mut discipline_guard = self.discipline.lock().await;
        tracing::debug!(target: "task::queue", %task, "push task");
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

    pub async fn pop_or_wait(&self) -> TxEnvelope {
        loop {
            let mut discipline_guard = self.discipline.lock().await;
            let bytes_opt = match &mut *discipline_guard {
                Discipline::Fcfs(queue) => {
                    queue.pop_front().map(|task| {
                        tracing::info!(target: "task::queue", %task, "pop task");
                        task.envelope
                    })
                }
                Discipline::Spt(queue) => {
                    queue.pop().map(|key| {
                        let task = key.0;
                        tracing::info!(target: "task::queue", %task, "pop task");
                        task.envelope
                    })
                }
                Discipline::Edf(queue) => {
                    queue.pop().map(|key| {
                        let task = key.0;
                        tracing::info!(target: "task::queue", %task, "pop task");
                        task.envelope
                    })
                }
                Discipline::SptEdf(queue) => {
                    queue.pop().map(|key| {
                        let task = key.0;
                        tracing::info!(target: "task::queue", %task, "pop task");
                        task.envelope
                    })
                }
                Discipline::EdfSpt(queue) => {
                    queue.pop().map(|key| {
                        let task = key.0;
                        tracing::info!(target: "task::queue", %task, "pop task");
                        task.envelope
                    })
                }
            };
            if let Some(envelope) = bytes_opt {
                return envelope;
            }
            drop(discipline_guard);
            self.wait().await;
        }
    }
}








