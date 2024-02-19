use super::*;
use airup_sdk::prelude::*;
use airupfx::prelude::*;
use std::sync::Arc;

#[derive(Debug)]
pub struct ReloadServiceHandle {
    helper: TaskHelperHandle,
}
impl TaskHandle for ReloadServiceHandle {
    fn task_class(&self) -> &'static str {
        "ReloadService"
    }

    fn is_important(&self) -> bool {
        false
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

    let reload_service = ReloadService { helper, context };
    reload_service.start();

    Arc::new(ReloadServiceHandle { helper: handle })
}

#[derive(Debug)]
struct ReloadService {
    helper: TaskHelper,
    context: Arc<SupervisorContext>,
}
impl ReloadService {
    fn start(mut self) {
        tokio::spawn(async move {
            let val = self.run().await;
            self.helper.finish(val);
        });
    }

    async fn run(&mut self) -> Result<(), Error> {
        if self.context.status.get() != Status::Active {
            return Err(Error::UnitNotStarted);
        }

        let service = &self.context.service;

        let ace = super::ace(&self.context).await?;

        if let Some(reload_cmd) = &service.exec.reload {
            ace.run_wait_timeout(reload_cmd, service.exec.reload_timeout())
                .await??;
        }

        Ok(())
    }
}
