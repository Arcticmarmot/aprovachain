use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};

#[derive(Debug, Clone)]
pub struct TaskQueue {
    pub inner: Arc<Mutex<Inner>>,
    pub notify: Arc<Notify>
}

#[derive(Debug, Clone)]
pub struct Inner {
    queue: VecDeque<Vec<u8>>
}

impl Inner {
    pub fn new() -> Self {
        Self { queue: VecDeque::new() }
    }
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::new())),
            notify: Arc::new(Notify::new())
        }
    }

    pub async fn push(&self, bytes: &[u8]) {
        let mut inner_guard = self.inner.lock().await;
        inner_guard.queue.push_back(Vec::from(bytes));
        tracing::info!(target: "task::queue", len=%inner_guard.queue.len(), "queue len");
        drop(inner_guard);
        self.notify.notify_one();
    }

    pub async fn pop(&self) -> Option<Vec<u8>> {
        let mut inner_guard = self.inner.lock().await;
        inner_guard.queue.pop_front()
    }

    pub async fn wait(&self) {
        self.notify.notified().await;
    }

    pub async fn len(&self) -> usize {
        let inner_guard = self.inner.lock().await;
        inner_guard.queue.len()
    }
}








