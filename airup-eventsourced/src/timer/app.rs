use super::timer::Timer;
use ahash::AHashMap;
use std::sync::OnceLock;
use tokio::time::Instant;

static TIMER_APP: OnceLock<TimerApp> = OnceLock::new();

#[derive(Debug)]
pub struct TimerApp {
    pub startup_time: Instant,
    pub persistent_time: Instant,
    pub timers: AHashMap<String, Timer>,
}

/// Gets a reference to the unique, global [`Timer`] instance.
///
/// # Panics
/// This method would panic if [`init`] was not previously called.
pub fn _timer_app() -> &'static TimerApp {
    TIMER_APP.get().unwrap()
}

/// Initializes the Timer app for use of [`timer`].
pub async fn init() -> anyhow::Result<()> {
    let object = TimerApp {
        startup_time: Instant::now(),
        persistent_time: Instant::now(),
        timers: AHashMap::with_capacity(16),
    };
    TIMER_APP.set(object).unwrap();
    Ok(())
}
