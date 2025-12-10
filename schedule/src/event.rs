use crate::metrics::{Score, MAX_SCORE};

pub const VALID_EVENT: Score = 10_000;
pub const TIMEOUT_EVENT: Score = 5_000;
pub const FAKE_RECEIPT_EVENT: Score = 4_000;

#[derive(Debug, Default)]
pub struct EventRecord {
    pub valid: u32,
    pub timeout: u32,
    pub fake_receipt: u32,
}

impl EventRecord {
    pub fn total_event_num(&self) -> u32 {
        self.valid + self.timeout + self.fake_receipt
    }
    pub fn compute_integrity_event(&self) -> Option<u128> {
        if self.total_event_num() == 0 { return None }
        let valid_event = self.valid as u128 * VALID_EVENT;
        let fake_receipt_event = self.fake_receipt as u128 * FAKE_RECEIPT_EVENT;
        let timeout_event = self.timeout as u128 * MAX_SCORE;
        let event = (valid_event + fake_receipt_event + timeout_event) / self.total_event_num() as u128;
        Some(event)
    }

    pub fn compute_timeliness_event(&self) -> Option<u128> {
        if self.total_event_num() == 0 { return None }
        let valid_event = self.valid as u128 * VALID_EVENT;
        let fake_receipt_event = self.fake_receipt as u128 * MAX_SCORE;
        let timeout_event = self.timeout as u128 * TIMEOUT_EVENT;
        let event = (valid_event + fake_receipt_event + timeout_event) / self.total_event_num() as u128;
        Some(event)
    }
}