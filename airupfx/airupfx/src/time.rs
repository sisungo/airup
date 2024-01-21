//! Utilities for tracking time.

use std::time::Duration;
use tokio::time::Instant;

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
    abort_handle: tokio::task::AbortHandle,
    reset_tx: tokio::sync::mpsc::Sender<()>,
    rx: tokio::sync::mpsc::Receiver<()>,
}
impl Alarm {
    pub fn new(dur: Duration) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        let (reset_tx, mut reset_rx) = tokio::sync::mpsc::channel(1);
        let join_handle = tokio::spawn(async move {
            let do_alarm = || async {
                tokio::time::sleep(dur).await;
                tx.send(()).await.ok();
            };
            loop {
                tokio::select! {
                    () = do_alarm() => {},
                    Some(()) = reset_rx.recv() => {},
                };
            }
        });

        Self {
            abort_handle: join_handle.abort_handle(),
            reset_tx,
            rx,
        }
    }

    pub fn reset(&mut self) {
        self.rx.try_recv().ok();
        self.reset_tx.try_send(()).ok();
    }

    pub async fn wait(&mut self) {
        self.rx.recv().await.unwrap()
    }
}
impl Drop for Alarm {
    fn drop(&mut self) {
        self.abort_handle.abort();
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
