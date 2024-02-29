use crate::app::airup_eventsourced;
use airup_sdk::files::timer::Timer as TimerDef;
use std::{sync::Arc, time::Duration};
use tokio::{task::JoinHandle, time::Instant};

#[derive(Debug)]
pub struct Timer {
    pub timer: Arc<TimerDef>,
    handle: JoinHandle<()>,
}
impl Timer {
    pub fn new(timer: TimerDef) -> Self {
        let timer = Arc::new(timer);
        let entity = TimerEntity::new(timer.clone());
        let handle = tokio::spawn(entity.run());

        Self { timer, handle }
    }
}
impl Drop for Timer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

struct TimerEntity {
    timer: Arc<TimerDef>,
}
impl TimerEntity {
    fn new(timer: Arc<TimerDef>) -> Self {
        Self { timer: timer }
    }

    async fn run(self) {
        let mut interval = tokio::time::interval_at(
            Instant::now(),
            Duration::from_millis(self.timer.timer.period.unwrap().get() as _),
        );
        loop {
            interval.tick().await;
            airup_eventsourced().run_command(self.timer.exec.command.clone());
        }
    }
}
