use std::collections::BTreeMap;
use account::executor::ExecutorId;

pub type Score = u128;
pub const MAX_SCORE: Score = 1_000_000;
// pub const EMA_K: u128 = 4;
pub type Metrics = BTreeMap<ExecutorId, (Integrity, Timeliness)>;

#[derive(Debug)]
pub struct Integrity {
    pub score: Score
}

impl Integrity {
    pub fn apply_event(&mut self, event: Score, ema_k: u128) {
        self.score = ema_new_val(self.score, event, ema_k);
    }
}

impl Default for Integrity {
    fn default() -> Self { Self { score: MAX_SCORE } }
}

#[derive(Debug)]
pub struct Timeliness {
    pub score: Score
}

impl Default for Timeliness {
    fn default() -> Self { Self { score: MAX_SCORE / 10000 } }
}

impl Timeliness {
    pub fn apply_event(&mut self, event: Score, ema_k: u128) {
        if self.score <= event {
            self.score = ema_new_val(self.score, event, ema_k * 64);
        } else {
            self.score = ema_new_val(self.score, event, ema_k);
        }
    }
}

/// 整数版本的 EMA
/// new = ((old * (K - 1)) + event) / K
pub fn ema_new_val(old_val: Score, event: Score, ema_k: u128) -> Score {
    let new_val = (old_val * (ema_k - 1)) + event;
    let round_new_val = (new_val + ema_k / 2) / ema_k;
    round_new_val
}