//! Utilities for tracking time.

use std::time::Duration;
use tokio::time::Instant;

/// A countdown timer.
pub struct Countdown {
    inst: Instant,
    dur: Option<Duration>,
}
impl Countdown {
    /// Creates a new [Countdown] instance with the given [Duration].
    pub fn new(dur: Option<Duration>) -> Self {
        Self {
            inst: Instant::now(),
            dur,
        }
    }

    /// Returns the time left until the timeout expired, returning [`Duration::ZERO`] if the timeout expired.
    #[inline]
    pub fn left(&self) -> Option<Duration> {
        self.dur.map(|x| x.saturating_sub(self.inst.elapsed()))
    }
}

/// Creates a countdown timer with given [Duration].
#[inline]
pub fn countdown(dur: Option<Duration>) -> Countdown {
    Countdown::new(dur)
}

/// Returns how many milliseconds passed since `1970-01-01 00:00:00`.
pub fn timestamp_ms() -> i64 {
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(x) => x.as_millis() as _,
        Err(err) => -(err.duration().as_millis() as i64),
    }
}
