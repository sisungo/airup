//! # The `CleanupService` Task

use super::*;
use crate::supervisor::SupervisorContext;
use airup_sdk::{files::Service, Error};
use airupfx::{ace::CommandExitError, prelude::*, process::Wait};
use std::sync::Arc;

#[derive(Debug)]
pub struct CleanupServiceHandle {
    helper: TaskHelperHandle,
    retry: bool,
}
impl CleanupServiceHandle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(context: Arc<SupervisorContext>, wait: Wait) -> Arc<dyn TaskHandle> {
        let (handle, helper) = task_helper();

        let retry_cond1 = context.service.retry.successful_exit || !wait.is_success();
        let retry_cond2 = context
            .retry
            .check_and_mark(context.service.retry.max_attempts);
        let retry = retry_cond1 && retry_cond2;

        let cleanup_service = CleanupService {
            helper,
            context,
            retry,
            wait,
        };
        cleanup_service.start();

        Arc::new(Self {
            helper: handle,
            retry,
        })
    }
}
impl TaskHandle for CleanupServiceHandle {
    fn task_type(&self) -> &'static str {
        match self.retry {
            true => "StartService",
            false => "StopService",
        }
    }

    fn send_interrupt(&self) {
        self.helper.send_interrupt()
    }

    fn wait(&self) -> BoxFuture<Result<TaskFeedback, Error>> {
        self.helper.wait()
    }
}

#[derive(Debug)]
struct CleanupService {
    helper: TaskHelper,
    context: Arc<SupervisorContext>,
    retry: bool,
    wait: Wait,
}
impl CleanupService {
    fn start(mut self) {
        tokio::spawn(async move {
            let val = self.run().await;
            self.helper.finish(val);
        });
    }

    async fn run(&mut self) -> Result<(), Error> {
        let ace = super::ace(&self.context).await?;

        cleanup_service(
            &ace,
            &self.context.service,
            &airupfx::time::countdown(self.context.service.exec.stop_timeout()),
        )
        .await
        .ok();

        if self.retry {
            super::StartServiceHandle::new(self.context.clone())
                .wait()
                .await?;
        } else if self.context.retry.enabled() && self.context.service.retry.successful_exit {
            self.context
                .last_error
                .set::<Error>(CommandExitError::from_wait_force(&self.wait).into());
        }

        Ok(())
    }
}

pub async fn cleanup_service(
    ace: &Ace,
    service: &Service,
    countdown: &airupfx::time::Countdown,
) -> Result<(), Error> {
    if let Some(x) = &service.service.pid_file {
        tokio::fs::remove_file(x).await.ok();
    }

    if let Some(x) = &service.exec.post_stop {
        for line in x.lines() {
            ace.run_wait_timeout(line.trim(), countdown.left())
                .await??;
        }
    }

    Ok(())
}
