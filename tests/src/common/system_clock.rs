use core::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

/// The system clock.
#[derive(Copy, Clone, Default)]
pub struct SystemClock;

impl sub60_cache::Clock for SystemClock {
    fn unix_ts(&self) -> Duration {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("we're past the UNIX epoch")
    }
}
