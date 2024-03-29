use super::runner::Timer;
use ahash::HashMap;
use airup_sdk::files::timer::Timer as TimerDef;
use std::sync::{Mutex, OnceLock};
use tokio::time::Instant;

static TIMER_APP: OnceLock<TimerApp> = OnceLock::new();

#[derive(Debug)]
pub struct TimerApp {
    pub _startup_time: Instant,
    pub _persistent_time: Instant,
    timers: Mutex<HashMap<String, Timer>>,
}
impl TimerApp {
    /// Load, reload or do nothing on the specified timer description.
    pub fn feed_timer(&self, name: String, new: TimerDef) {
        let mut timers = self.timers.lock().unwrap();
        if let Some(timer) = timers.get_mut(&name) {
            if *timer.timer == new {
                *timer = Timer::new(new);
            }
        } else {
            timers.insert(name, Timer::new(new));
        }
    }

    /// Call [`HashMap::retain`] on timers.
    pub fn retain_timers(&self, f: impl Fn(&String) -> bool) {
        self.timers.lock().unwrap().retain(|key, _| f(key));
    }
}

/// Gets a reference to the unique, global [`Timer`] instance.
///
/// # Panics
/// This method would panic if [`init`] was not previously called.
pub fn timer_app() -> &'static TimerApp {
    TIMER_APP.get().unwrap()
}

/// Initializes the Timer app for use of [`timer_app`].
pub async fn init() -> anyhow::Result<()> {
    let object = TimerApp {
        _startup_time: Instant::now(),
        _persistent_time: Instant::now(),
        timers: HashMap::with_capacity_and_hasher(16, ahash::RandomState::new()).into(),
    };
    TIMER_APP.set(object).unwrap();
    Ok(())
}
