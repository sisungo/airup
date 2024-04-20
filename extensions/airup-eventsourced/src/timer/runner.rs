use airup_sdk::files::timer::Timer as TimerDef;
use airupfx_io::line_piper::{self, Callback as LinePiperCallback};
use anyhow::anyhow;
use std::{future::Future, pin::Pin, sync::Arc, time::Duration};
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
        Self { timer }
    }

    async fn run(self) {
        let mut interval = tokio::time::interval_at(
            Instant::now(),
            Duration::from_millis(self.timer.timer.period.unwrap().get() as _),
        );
        loop {
            interval.tick().await;
            run_command(&self.timer).await.ok();
        }
    }
}

async fn run_command(def: &TimerDef) -> anyhow::Result<()> {
    #[cfg(target_family = "unix")]
    let mut child = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(&def.exec.command)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("stdout not piped"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("stderr not piped"))?;
    let name = format!("airup_timer_{}", def.name);
    line_piper::set_callback(stdout, Box::new(LogCallback::new(name.clone(), "stdout")));
    line_piper::set_callback(stderr, Box::new(LogCallback::new(name.clone(), "stderr")));

    child.wait().await?;

    Ok(())
}

#[derive(Clone)]
struct LogCallback {
    name: String,
    module: &'static str,
}
impl LogCallback {
    pub fn new(name: String, module: &'static str) -> Self {
        Self { name, module }
    }
}
impl LinePiperCallback for LogCallback {
    fn invoke<'a>(
        &'a self,
        msg: &'a [u8],
    ) -> Pin<Box<dyn for<'b> Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            crate::app::airup_eventsourced()
                .append_log(&self.name, self.module, msg)
                .await;
        })
    }
    fn clone_boxed(&self) -> Box<dyn LinePiperCallback> {
        Box::new(self.clone())
    }
}
