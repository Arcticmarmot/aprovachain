use std::collections::BTreeMap;
use account::executor::ExecutorId;

pub type Score = u128;
pub const MAX_SCORE: Score = 10_000;
pub const EMA_K: u128 = 4;
pub type Metrics = BTreeMap<ExecutorId, (Integrity, Timeliness)>;

#[derive(Debug)]
pub struct Integrity {
    pub score: Score
}

impl Integrity {
    pub fn apply_event(&mut self, event: Score) {
        self.score = ema_new_val(self.score, event);
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
    fn default() -> Self { Self { score: MAX_SCORE / 2 } }
}

impl Timeliness {
    pub fn apply_event(&mut self, event: Score) {
        self.score = ema_new_val(self.score, event);
    }
}

/// 整数版本的 EMA
/// new = ((old * (K - 1)) + event) / K
pub fn ema_new_val(old_val: Score, event: Score) -> Score {
    let new_val = (old_val * (EMA_K - 1)) + event;
    let round_new_val = (new_val + EMA_K / 2) / EMA_K;
    round_new_val
}