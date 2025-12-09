use std::collections::BTreeMap;
use account::executor::ExecutorId;
use tx::id::TxEnvelopeId;

pub type Score = u128;
pub type Weight = u128;
pub const MAX_SCORE: Score = 10_000;
pub const VALID_EVENT: Score = 10_000;
pub const TIMEOUT_EVENT: Score = 5_000;
pub const FAKE_RECEIPT_EVENT: Score = 4_000;
const EMA_K: u128 = 8;
pub type Metrics = BTreeMap<ExecutorId, (Integrity, Timeliness)>;

pub struct BlockEventRecord {
    pub valid: u32,
    pub timeout: u32,
    pub fake_receipt: u32,
}

impl Default for BlockEventRecord {
    fn default() -> Self {
        Self {
            valid: 0,
            timeout: 0,
            fake_receipt: 0,
        }
    }
}

impl BlockEventRecord {
    pub fn compute_integrity_event(&self) -> u128 {
        let valid_event = self.valid as u128 * VALID_EVENT;
        let fake_receipt_event = self.fake_receipt as u128 * FAKE_RECEIPT_EVENT;
        let timeout_event = self.timeout as u128 * MAX_SCORE;
        (valid_event + fake_receipt_event + timeout_event) / self.total_event_num() as u128
    }

    pub fn compute_timeliness_event(&self) -> u128 {
        let valid_event = self.valid as u128 * VALID_EVENT;
        let fake_receipt_event = self.fake_receipt as u128 * MAX_SCORE;
        let timeout_event = self.timeout as u128 * TIMEOUT_EVENT;
        (valid_event + fake_receipt_event + timeout_event) / self.total_event_num() as u128
    }

    pub fn total_event_num(&self) -> u32 {
        self.valid + self.timeout + self.fake_receipt
    }
}

#[derive(Debug)]
pub struct Integrity {
    pub score: Score
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

/// 整数版本的 EMA
/// new = ((old * (K - 1)) + event) / K
pub fn ema_new_val(old_val: Score, event: Score) -> Score {
    let new_val = (old_val * (EMA_K - 1)) + event;
    let round_new_val = (new_val + EMA_K / 2) / EMA_K;
    round_new_val
}

/// 根据 integrity 和 timeliness 计算出权重
pub fn weight_from_metrics(integrity: &Integrity, timeliness: &Timeliness) -> Weight {
    let igt_weight = integrity.score * 60 / 100;
    let tln_weight = timeliness.score * 40 / 100;
    let weight = igt_weight + tln_weight;
    weight.max(1)
}

/// 根据 tx_id 的哈希生成随机数
pub fn pseudo_random_u128(tx_id: &TxEnvelopeId) -> u128 {
    let mut out = [0u8; 16];
    out.copy_from_slice(&tx_id.hash()[0..16]);
    u128::from_be_bytes(out)
}