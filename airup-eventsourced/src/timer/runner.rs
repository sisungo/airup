use crate::app::airup_eventsourced;
use airup_sdk::files::timer::Timer as TimerDef;
use std::sync::Arc;
use tokio::task::JoinHandle;

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
    _timer: Arc<TimerDef>,
}
impl TimerEntity {
    fn new(timer: Arc<TimerDef>) -> Self {
        Self { _timer: timer }
    }

    async fn run(self) {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            airup_eventsourced().run_command("echo Test Success".into())
        }
    }
}
