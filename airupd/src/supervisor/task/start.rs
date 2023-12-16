use super::*;
use crate::{app::airupd, supervisor::SupervisorContext};
use airup_sdk::{files::service::Kind, system::Status, Error};
use airupfx::prelude::*;
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
    fn task_class(&self) -> &'static str {
        "StartService"
    }

    fn task_name(&self) -> &'static str {
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
    fn start(mut self) {
        tokio::spawn(async move {
            let val = self.run().await;
            self.helper.finish(val);
        });
    }

    async fn run(&mut self) -> Result<(), Error> {
        // The task immediately fails if the service is already active
        if self.context.status.get() == Status::Active {
            return Err(Error::UnitStarted);
        }

        // Auto saving of last error is enabled for this task
        self.context.last_error.set(None);
        self.context.last_error.set_autosave(true);

        let ace = super::ace(&self.context).await?;

        self.helper.would_interrupt(self.solve_deps()).await??;

        let countdown = airupfx::time::countdown(self.context.service.exec.start_timeout());

        if let Some(x) = &self.context.service.exec.pre_start {
            for line in x.lines() {
                ace.run_wait_timeout(line.trim(), countdown.left())
                    .await??;
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
                ace.run_wait_timeout(&self.context.service.exec.start, countdown.left())
                    .await??;

                let pid: i64 = tokio::fs::read_to_string(
                    &self.context.service.service.pid_file.as_ref().unwrap(),
                )
                .await
                .map_err(Error::pid_file)?
                .trim()
                .parse()
                .map_err(Error::pid_file)?;

                let child = airupfx::ace::Child::Process(
                    airupfx::process::Child::from_pid(pid).map_err(|err| Error::Io {
                        message: err.to_string(),
                    })?,
                );
                self.context.set_child(child).await;
            }
            Kind::Oneshot => {
                ace.run_wait_timeout(&self.context.service.exec.start, countdown.left())
                    .await??;
            }
            Kind::Notify => {
                // TODO: Implement `notify`
                self.context
                    .set_child(ace.run(&self.context.service.exec.start).await?)
                    .await;
            }
        }

        if let Some(x) = &self.context.service.exec.post_start {
            for line in x.lines() {
                ace.run_wait_timeout(line.trim(), countdown.left())
                    .await??;
            }
        }

        self.context.status.set(Status::Active);

        Ok(())
    }

    async fn solve_deps(&self) -> Result<(), Error> {
        // If any of conflict services are active, the task fails
        for i in self.context.service.service.conflicts_with.iter() {
            if let Some(handle) = airupd().supervisors.get(i).await {
                if handle.query().await.status == Status::Active {
                    return Err(Error::ConflictsWith {
                        name: i.to_string(),
                    });
                }
            }
        }

        // Start dependencies
        for dep in self.context.service.service.dependencies.iter() {
            airupd()
                .make_service_active(dep)
                .await
                .map_err(|_| Error::dep_not_satisfied(dep))?;
        }

        Ok(())
    }
}
