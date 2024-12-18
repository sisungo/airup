//! # The `CleanupService` Task
//! This is started when the service stopped. This executes the cleanup command, and (if necessary) restarts the service.
//! Depending on what it is doing, it may act as `StartService` or `StopService`.

use super::*;
use crate::ace::CommandExitError;
use airup_sdk::files::Service;
use airupfx::{prelude::*, process::Wait};
use std::{
    sync::{
        Arc,
        atomic::{self, AtomicBool},
    },
    time::Duration,
};

#[derive(Debug)]
pub struct CleanupServiceHandle {
    helper: TaskHelperHandle,
    important: Arc<AtomicBool>,
    retry: bool,
}
impl TaskHandle for CleanupServiceHandle {
    fn task_class(&self) -> &'static str {
        match self.retry {
            true => "StartService",
            false => "StopService",
        }
    }

    fn is_important(&self) -> bool {
        self.important.load(atomic::Ordering::Acquire)
    }

    fn send_interrupt(&self) {
        self.helper.send_interrupt()
    }

    fn wait(&self) -> BoxFuture<Result<TaskFeedback, Error>> {
        self.helper.wait()
    }
}

pub(in crate::supervisor) fn start(
    context: Arc<SupervisorContext>,
    wait: Wait,
) -> Arc<dyn TaskHandle> {
    let (handle, helper) = task_helper();
    let important: Arc<AtomicBool> = AtomicBool::new(true).into();

    let retry_cond1 = context.service.watchdog.successful_exit || !wait.is_success();
    let retry_cond2 = context
        .retry
        .check_and_mark(context.service.retry.max_attempts);
    let retry = retry_cond1 && retry_cond2;

    let cleanup_service = CleanupService {
        helper,
        context,
        important: important.clone(),
        retry,
        wait,
    };
    cleanup_service.start();

    Arc::new(CleanupServiceHandle {
        helper: handle,
        important,
        retry,
    })
}

#[derive(Debug)]
struct CleanupService {
    helper: TaskHelper,
    context: Arc<SupervisorContext>,
    important: Arc<AtomicBool>,
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

        _ = cleanup_service(
            &ace,
            &self.context.service,
            &airupfx::time::countdown(self.context.service.exec.stop_timeout()),
        )
        .await;

        self.important.store(false, atomic::Ordering::Release);
        self.helper
            .would_interrupt(async {
                tokio::time::sleep(Duration::from_millis(self.context.service.retry.delay)).await;
            })
            .await?;

        if self.retry {
            self.important.store(true, atomic::Ordering::Release);
            let handle = super::start::start(self.context.clone());
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
        _ = tokio::fs::remove_file(x).await;
    }

    if let Some(x) = &service.exec.post_stop {
        for line in x.lines() {
            ace.run_wait_timeout(line.trim(), countdown.left())
                .await??;
        }
    }

    Ok(())
}
