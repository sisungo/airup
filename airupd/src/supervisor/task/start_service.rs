use super::*;
use crate::{
    app::airupd,
    supervisor::{AirupdExt, SupervisorContext},
};
use airup_sdk::{system::Status, Error};
use airupfx::{files::service::Kind, prelude::*};
use std::sync::Arc;

#[derive(Debug)]
pub struct StartServiceHandle {
    helper: TaskHelperHandle,
}
impl StartServiceHandle {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(context: Arc<SupervisorContext>) -> Arc<dyn TaskHandle> {
        let (handle, helper) = task_helper();

        let start_service = StartService { helper, context };
        start_service.start();

        Arc::new(Self { helper: handle })
    }
}
impl TaskHandle for StartServiceHandle {
    fn task_type(&self) -> &'static str {
        "StartService"
    }

    fn send_interrupt(&self) {
        self.helper.send_interrupt()
    }

    fn wait(&self) -> BoxFuture<Result<TaskFeedback, Error>> {
        self.helper.wait()
    }
}

#[derive(Debug)]
struct StartService {
    helper: TaskHelper,
    context: Arc<SupervisorContext>,
}
impl StartService {
    pub fn start(mut self) {
        tokio::spawn(async move {
            let val = self.run().await;
            self.helper.finish(val);
        });
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        if self.context.status.get() == Status::Active {
            return Err(Error::UnitStarted);
        }

        self.context.last_error.set(None);
        self.context.last_error.set_autosave(true);

        let ace = super::ace(&self.context).await?;

        for i in self.context.service.service.conflicts_with.iter() {
            if let Some(handle) = airupd().supervisors.get(i).await {
                if handle.query().await.status == Status::Active {
                    return Err(Error::ConflictsWith {
                        name: i.to_string(),
                    });
                }
            }
        }

        self.helper
            .interruptable_scope::<Result<(), Error>, _>(async {
                for dep in self.context.service.service.dependencies.iter() {
                    airupd()
                        .make_service_active(dep)
                        .await
                        .map_err(|_| Error::dependency_not_satisfied(dep))?;
                }

                Ok(())
            })
            .await??;

        let countdown = airupfx::time::countdown(self.context.service.exec.start_timeout());

        if let Some(x) = &self.context.service.exec.pre_start {
            for line in x.lines() {
                ace.run_timeout(line.trim(), countdown.left()).await??;
            }
        }

        match self.context.service.service.kind {
            Kind::Simple => {
                self.context
                    .set_child(ace.run(&self.context.service.exec.start).await?)
                    .await;

                if let Some(pid_file) = &self.context.service.service.pid_file {
                    tokio::fs::write(pid_file, self.context.pid().await.unwrap().to_string())
                        .await
                        .ok();
                }
            }
            Kind::Forking => {
                let _lock = airupfx::process::prepare_ops().await;
                ace.run_timeout(&self.context.service.exec.start, countdown.left())
                    .await??;
                let pid: Pid = tokio::fs::read_to_string(
                    &self.context.service.service.pid_file.as_ref().unwrap(),
                )
                .await
                .map_err(Error::pid_file)?
                .trim()
                .parse()
                .map_err(Error::pid_file)?;
                self.context
                    .set_child(airupfx::ace::Child::Process(
                        airupfx::process::Child::from_pid(pid)
                            .await
                            .map_err(|err| Error::io(&err))?,
                    ))
                    .await;
            }
            Kind::Oneshot => {
                ace.run_timeout(&self.context.service.exec.start, countdown.left())
                    .await??;
            }
            Kind::Notify => {}
        }

        if let Some(x) = &self.context.service.exec.post_start {
            for line in x.lines() {
                ace.run_timeout(line.trim(), countdown.left()).await??;
            }
        }

        self.context.status.set(Status::Active);

        Ok(())
    }
}
