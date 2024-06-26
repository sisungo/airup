use super::*;
use crate::ace::CommandExitError;
use airupfx::prelude::*;
use std::{sync::Arc, time::Duration};

#[derive(Debug)]
pub struct HealthCheckHandle {
    helper: TaskHelperHandle,
}
impl TaskHandle for HealthCheckHandle {
    fn task_class(&self) -> &'static str {
        "HealthCheck"
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

pub(in crate::supervisor) async fn start(context: &SupervisorContext) -> Arc<dyn TaskHandle> {
    let (handle, helper) = task_helper();
    let command = context.service.exec.health_check.clone();
    let timeout = context.service.exec.health_check_timeout();

    let reload_service = HealthCheck {
        helper,
        ace: super::ace(context).await,
        command,
        timeout,
    };
    reload_service.start();

    Arc::new(HealthCheckHandle { helper: handle })
}

struct HealthCheck {
    helper: TaskHelper,
    ace: Result<Ace, Error>,
    command: Option<String>,
    timeout: Option<Duration>,
}
impl HealthCheck {
    fn start(mut self) {
        tokio::spawn(async move {
            let val = self.run().await;
            self.helper.finish(val);
        });
    }

    async fn run(&mut self) -> Result<(), Error> {
        let ace = std::mem::replace(&mut self.ace, Err(Error::internal("taken ace")))?;
        self.helper
            .would_interrupt(async {
                if let Some(x) = &self.command {
                    ace.run_wait_timeout(x, self.timeout).await??;
                }
                Ok::<_, Error>(Ok::<_, CommandExitError>(()))
            })
            .await???;
        Ok(())
    }
}
