use super::*;
use crate::supervisor::SupervisorContext;
use airup_sdk::{files::service::Kind, system::Status, Error};
use airupfx::{signal::SIGTERM, util::BoxFuture};
use std::sync::Arc;

#[derive(Debug)]
pub struct StopServiceHandle {
    helper: TaskHelperHandle,
}
impl StopServiceHandle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(context: Arc<SupervisorContext>) -> Arc<dyn TaskHandle> {
        let (handle, helper) = task_helper();

        let stop_service = StopService { helper, context };
        stop_service.start();

        Arc::new(Self { helper: handle })
    }
}
impl TaskHandle for StopServiceHandle {
    fn task_type(&self) -> &'static str {
        "StopService"
    }

    fn send_interrupt(&self) {
        self.helper.send_interrupt()
    }

    fn wait(&self) -> BoxFuture<Result<TaskFeedback, Error>> {
        self.helper.wait()
    }
}

#[derive(Debug)]
struct StopService {
    helper: TaskHelper,
    context: Arc<SupervisorContext>,
}
impl StopService {
    pub fn start(mut self) {
        tokio::spawn(async move {
            let val = self.run().await;
            self.helper.finish(val);
        });
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        let service = &self.context.service;

        if self.context.status.get() != Status::Active {
            return Err(Error::UnitNotStarted);
        }

        self.context.last_error.set(None);

        let ace = super::ace(&self.context).await?;

        let countdown = airupfx::time::countdown(service.exec.stop_timeout());

        if let Some(x) = &self.context.service.exec.pre_stop {
            for line in x.lines() {
                ace.run_wait_timeout(line.trim(), countdown.left())
                    .await??;
            }
        }

        match &service.exec.stop {
            Some(x) => {
                ace.run_wait_timeout(x, self.context.service.exec.stop_timeout()).await??;
            }
            None => {
                if let Some(x) = self.context.child.write().await.as_mut() {
                    x.kill_timeout(SIGTERM, countdown.left()).await?;
                } else {
                    return Err(Error::unsupported("this service cannot be stopped"));
                }
            }
        };

        if let Some(x) = &self.context.service.service.pid_file {
            if self.context.service.service.kind == Kind::Simple {
                tokio::fs::remove_file(x).await.ok();
            }
        }

        self.context.status.set(Status::Stopped);

        super::cleanup_service(&ace, &self.context.service, &countdown)
            .await
            .ok();

        Ok(())
    }
}
