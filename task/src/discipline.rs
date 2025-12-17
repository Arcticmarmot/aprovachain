use std::cmp::{Ordering};
use std::collections::{BinaryHeap, VecDeque};

#[derive(Debug, Clone)]
pub struct Task {
    pub bytes: Vec<u8>,
    pub scale: u32,
    pub deadline: u128,
    pub seq: u64,
}

#[derive(Debug, Clone)]
pub enum Discipline {
    Fcfs(VecDeque<Task>),
    Spt(BinaryHeap<SptKey>), // Shortest Processing Time
    Edf(BinaryHeap<EdfKey>),
    SptEdf(BinaryHeap<SptEdfKey>),
    EdfSpt(BinaryHeap<EdfSptKey>),
}

impl Discipline {
    pub fn len(&self) -> usize {
        match self {
            Discipline::Fcfs(queue) => {
                queue.len()
            },
            Discipline::Spt(queue) => {
                queue.len()
            }
            Discipline::Edf(queue) => {
                queue.len()
            }
            Discipline::SptEdf(queue) => {
                queue.len()
            }
            Discipline::EdfSpt(queue) => {
                queue.len()
            }
        }
    }
}


#[derive(Debug, Clone)]
pub struct SptKey(pub Task);
impl Eq for SptKey { }

impl PartialEq for SptKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.scale == other.0.scale && self.0.seq == other.0.seq
    }
}

impl Ord for SptKey {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.scale.cmp(&self.0.scale)
            .then_with(|| other.0.seq.cmp(&self.0.seq))
    }
}

impl PartialOrd for SptKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct EdfKey(pub Task);

impl Eq for EdfKey {}
impl PartialEq for EdfKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.deadline == other.0.deadline && self.0.seq == other.0.seq
    }
}

impl Ord for EdfKey {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.deadline.cmp(&self.0.deadline)
            .then_with(|| other.0.seq.cmp(&self.0.seq))
    }
}

impl PartialOrd for EdfKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct SptEdfKey(pub Task);

impl Eq for SptEdfKey { }
impl PartialEq for SptEdfKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.scale == other.0.scale &&
            self.0.deadline == other.0.deadline &&
            self.0.seq == other.0.seq
    }
}

impl Ord for SptEdfKey {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.scale.cmp(&self.0.scale)
            .then_with(|| other.0.deadline.cmp(&self.0.deadline))
            .then_with(|| other.0.seq.cmp(&self.0.seq))
    }
}

impl PartialOrd for SptEdfKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone)]
pub struct EdfSptKey(pub Task);

impl Eq for EdfSptKey { }
impl PartialEq for EdfSptKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.deadline == other.0.deadline &&
            self.0.scale == other.0.scale &&
            self.0.seq == other.0.seq
    }
}

impl Ord for EdfSptKey {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.deadline.cmp(&self.0.deadline)
            .then_with(|| other.0.scale.cmp(&self.0.scale))
            .then_with(|| other.0.seq.cmp(&self.0.seq))
    }
}

impl PartialOrd for EdfSptKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}