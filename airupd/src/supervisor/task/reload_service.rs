use super::{task_helper, TaskFeedback, TaskHandle, TaskHelper, TaskHelperHandle};
use crate::supervisor::SupervisorContext;
use airupfx::{
    prelude::*,
    sdk::{system::Status, Error},
};
use std::sync::Arc;

#[derive(Debug)]
pub struct ReloadServiceHandle {
    helper: TaskHelperHandle,
}
impl ReloadServiceHandle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(context: Arc<SupervisorContext>) -> Arc<dyn TaskHandle> {
        let (handle, helper) = task_helper();

        let reload_service = ReloadService { helper, context };
        reload_service.start();

        Arc::new(Self { helper: handle })
    }
}
impl TaskHandle for ReloadServiceHandle {
    fn task_type(&self) -> &'static str {
        "ReloadService"
    }

    fn send_interrupt(&self) {
        self.helper.send_interrupt()
    }

    fn wait(&self) -> BoxFuture<Result<TaskFeedback, Error>> {
        self.helper.wait()
    }
}

#[derive(Debug)]
struct ReloadService {
    helper: TaskHelper,
    context: Arc<SupervisorContext>,
}
impl ReloadService {
    pub fn start(mut self) {
        tokio::spawn(async move {
            let val = self.run().await;
            self.helper.finish(val);
        });
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        if self.context.status() != Status::Active {
            return Err(Error::ObjectNotConfigured);
        }

        let service = &self.context.service;

        let ace = super::ace(&self.context).await?;

        if let Some(reload_cmd) = &service.exec.reload {
            ace.run_timeout(reload_cmd, service.exec.reload_timeout())
                .await??;
        }

        Ok(())
    }
}
