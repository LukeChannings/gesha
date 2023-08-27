use std::time::{SystemTime, UNIX_EPOCH, SystemTimeError};

pub fn get_unix_timestamp(time: SystemTime) -> Result<i64, SystemTimeError> {
    return Ok(time.duration_since(UNIX_EPOCH)?.as_millis() as i64);
}
