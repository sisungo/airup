use super::*;
use airup_sdk::system::Status;
use airupfx::signal::SIGTERM;
use std::sync::Arc;

#[derive(Debug)]
pub struct StopServiceHandle {
    helper: TaskHelperHandle,
}
impl TaskHandle for StopServiceHandle {
    fn task_class(&self) -> &'static str {
        "StopService"
    }

    fn is_important(&self) -> bool {
        true
    }

    fn send_interrupt(&self) {
        self.helper.send_interrupt()
    }

    fn wait(&self) -> BoxFuture<Result<TaskFeedback, Error>> {
        self.helper.wait()
    }
}

pub fn start(context: Arc<SupervisorContext>) -> Arc<dyn TaskHandle> {
    let (handle, helper) = task_helper();

    let stop_service = StopService { helper, context };
    stop_service.start();

    Arc::new(StopServiceHandle { helper: handle })
}

#[derive(Debug)]
struct StopService {
    helper: TaskHelper,
    context: Arc<SupervisorContext>,
}
impl StopService {
    fn start(mut self) {
        tokio::spawn(async move {
            let val = self.run().await;
            self.helper.finish(val);
        });
    }

    async fn run(&mut self) -> Result<(), Error> {
        // The task immediately fails if the service is not active
        if self.context.status.get() != Status::Active {
            return Err(Error::UnitNotStarted);
        }

        // Auto saving of last error is disabled for this task
        self.context.last_error.set(None);

        let ace = super::ace(&self.context).await?;
        let countdown = airupfx::time::countdown(self.context.service.exec.stop_timeout());

        if let Some(x) = &self.context.service.exec.pre_stop {
            for line in x.lines() {
                ace.run_wait_timeout(line.trim(), countdown.left())
                    .await??;
            }
        }

        match &self.context.service.exec.stop {
            Some(x) => {
                ace.run_wait_timeout(x, countdown.left()).await??;
            }
            None => {
                if let Some(x) = self.context.child.write().await.as_mut() {
                    x.kill_timeout(SIGTERM, countdown.left()).await?;
                } else {
                    return Err(Error::unsupported("this service cannot be stopped"));
                }
            }
        };

        self.context.status.set(Status::Stopped);

        super::cleanup::cleanup_service(&ace, &self.context.service, &countdown)
            .await
            .ok();

        Ok(())
    }
}
