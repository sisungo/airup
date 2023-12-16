//! # The `CleanupService` Task

use super::*;
use crate::supervisor::SupervisorContext;
use airup_sdk::{files::Service, Error};
use airupfx::{ace::CommandExitError, prelude::*, process::Wait};
use std::{
    sync::{
        atomic::{self, AtomicBool},
        Arc,
    },
    time::Duration,
};

#[derive(Debug)]
pub struct CleanupServiceHandle {
    helper: TaskHelperHandle,
    is_retrying: Arc<AtomicBool>,
    retry: bool,
}
impl CleanupServiceHandle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(context: Arc<SupervisorContext>, wait: Wait) -> Arc<dyn TaskHandle> {
        let (handle, helper) = task_helper();
        let is_retrying: Arc<AtomicBool> = Arc::default();

        let retry_cond1 = context.service.watchdog.successful_exit || !wait.is_success();
        let retry_cond2 = context
            .retry
            .check_and_mark(context.service.retry.max_attempts);
        let retry = retry_cond1 && retry_cond2;

        let cleanup_service = CleanupService {
            helper,
            context,
            is_retrying: is_retrying.clone(),
            retry,
            wait,
        };
        cleanup_service.start();

        Arc::new(Self {
            helper: handle,
            is_retrying,
            retry,
        })
    }
}
impl TaskHandle for CleanupServiceHandle {
    fn task_class(&self) -> &'static str {
        match self.retry {
            true => "StartService",
            false => "StopService",
        }
    }

    fn task_name(&self) -> &'static str {
        if self.is_retrying.load(atomic::Ordering::SeqCst) {
            "StartService"
        } else {
            "CleanupService"
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
    is_retrying: Arc<AtomicBool>,
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

        self.helper
            .would_interrupt(async {
                tokio::time::sleep(Duration::from_millis(self.context.service.retry.delay)).await;
            })
            .await?;

        cleanup_service(
            &ace,
            &self.context.service,
            &airupfx::time::countdown(self.context.service.exec.stop_timeout()),
        )
        .await
        .ok();

        if self.retry {
            self.is_retrying.store(true, atomic::Ordering::SeqCst);
            let handle = super::StartServiceHandle::new(self.context.clone());
            tokio::select! {
                _ = handle.wait() => {},
                _ = self.helper.interrupt_flag.wait_for(|x| *x) => {
                    handle.send_interrupt();
                },
            };
        } else if self.context.retry.enabled() && self.context.service.watchdog.successful_exit {
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
