//! Utilities for tracking time.

use std::time::Duration;
use tokio::time::{Instant, Interval};

/// A countdown timer.
#[derive(Debug)]
pub struct Countdown {
    inst: Instant,
    dur: Option<Duration>,
}
impl Countdown {
    /// Creates a new [`Countdown`] instance with the given [`Duration`].
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

/// An alarm timer.
#[derive(Debug)]
pub struct Alarm {
    dur: Duration,
    interval: Option<Interval>,
}
impl Alarm {
    pub fn new(dur: Duration) -> Self {
        Self {
            dur,
            interval: Some(tokio::time::interval(dur)),
        }
    }

    pub fn enable(&mut self) {
        self.interval = Some(tokio::time::interval(self.dur));
    }

    pub fn disable(&mut self) {
        self.interval = None;
    }

    pub async fn wait(&mut self) -> Option<()> {
        match &mut self.interval {
            Some(x) => {
                x.tick().await;
                Some(())
            }
            None => None,
        }
    }
}

/// Creates a countdown timer with given [`Duration`].
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
