use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, VecDeque};

#[derive(Debug, Clone)]
pub struct Task {
    pub bytes: Vec<u8>,
    pub scale: u32
}

#[derive(Debug, Clone)]
pub enum Strategy {
    Fifo(VecDeque<Task>),
    SmallestFirst(BinaryHeap<Reverse<BySmallestTask>>),
}

impl Strategy {
    pub fn len(&self) -> usize {
        match self {
            Strategy::Fifo(queue) => {
                queue.len()
            },
            Strategy::SmallestFirst(queue) => {
                queue.len()
            }
        }
    }
}


#[derive(Debug, Clone)]
pub struct BySmallestTask(pub Task);

impl Eq for BySmallestTask { }
impl PartialEq for BySmallestTask {
    fn eq(&self, other: &Self) -> bool { self.0.scale == other.0.scale }
}
impl Ord for BySmallestTask {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_scale = self.0.scale;
        let other_scale = other.0.scale;
        self_scale.cmp(&other_scale)
    }
}
impl PartialOrd for BySmallestTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}