use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};

pub fn unix_time_secs() -> Result<u64, SystemTimeError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    Ok(now.as_secs())
}
