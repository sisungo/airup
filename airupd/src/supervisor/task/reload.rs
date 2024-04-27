use super::*;
use crate::ace::CommandExitError;
use airup_sdk::prelude::*;
use airupfx::prelude::*;
use std::{sync::Arc, time::Duration};

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

pub(in crate::supervisor) async fn start(context: &SupervisorContext) -> Arc<dyn TaskHandle> {
    let (handle, helper) = task_helper();

    let reload_service = ReloadService {
        helper,
        ace: super::ace(context).await,
        status: context.status.get(),
        reload_cmd: context.service.exec.reload.clone(),
        reload_timeout: context.service.exec.reload_timeout(),
    };
    reload_service.start();

    Arc::new(ReloadServiceHandle { helper: handle })
}

struct ReloadService {
    helper: TaskHelper,
    ace: Result<Ace, Error>,
    status: Status,
    reload_cmd: Option<String>,
    reload_timeout: Option<Duration>,
}
impl ReloadService {
    fn start(mut self) {
        tokio::spawn(async move {
            let val = self.run().await;
            self.helper.finish(val);
        });
    }

    async fn run(&mut self) -> Result<(), Error> {
        if self.status != Status::Active {
            return Err(Error::NotStarted);
        }

        let ace = std::mem::replace(&mut self.ace, Err(Error::internal("taken ace")))?;

        self.helper
            .would_interrupt(async {
                if let Some(reload_cmd) = &self.reload_cmd {
                    ace.run_wait_timeout(reload_cmd, self.reload_timeout)
                        .await??;
                }
                Ok::<_, Error>(Ok::<_, CommandExitError>(()))
            })
            .await???;

        Ok(())
    }
}
