use super::*;
use crate::app::airupd;
use airup_sdk::{files::service::Kind, system::Status};
use airupfx::prelude::*;
use std::sync::Arc;

#[derive(Debug)]
pub struct StartServiceHandle {
    helper: TaskHelperHandle,
}
impl TaskHandle for StartServiceHandle {
    fn task_class(&self) -> &'static str {
        "StartService"
    }

    fn is_important(&self) -> bool {
        true
    }

    fn send_interrupt(&self) {
        self.helper.send_interrupt()
    }

    fn wait(&self) -> BoxFuture<Result<TaskFeedback, Error>> {
        self.helper.wait()
    }
}

pub(in crate::supervisor) fn start(context: Arc<SupervisorContext>) -> Arc<dyn TaskHandle> {
    let (handle, helper) = task_helper();

    let start_service = StartService { helper, context };
    start_service.start();

    Arc::new(StartServiceHandle { helper: handle })
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
            return Err(Error::Started);
        }

        // The task immediately fails if the service is `forking`-kinded, and supervising it is not supported on the system
        if self.context.service.service.kind == Kind::Forking
            && !airupfx::process::is_forking_supervisable()
        {
            return Err(Error::unsupported(
                "supervising `forking`-kinded services are unsupported on the system",
            ));
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
                self.start_simple(&ace).await?;
            }
            Kind::Forking => {
                self.start_forking(&ace, &countdown).await?;
            }
            Kind::Oneshot => {
                ace.run_wait_timeout(&self.context.service.exec.start, countdown.left())
                    .await??;
            }
            Kind::Notify => {
                self.start_notify(&ace, &countdown).await?;
            }
        }

        if let Some(x) = &self.context.service.exec.post_start {
            for line in x.lines() {
                ace.run_wait_timeout(line.trim(), countdown.left())
                    .await
                    .ok();
            }
        }

        self.context.status.set(Status::Active);

        Ok(())
    }

    async fn start_forking(&mut self, ace: &Ace, countdown: &Countdown) -> Result<(), Error> {
        ace.run_wait_timeout(&self.context.service.exec.start, countdown.left())
            .await??;

        let pid: i64 =
            tokio::fs::read_to_string(&self.context.service.service.pid_file.as_ref().unwrap())
                .await
                .map_err(Error::pid_file)?
                .trim()
                .parse()
                .map_err(Error::pid_file)?;

        let child = airupfx::ace::Child::Process(airupfx::process::Child::from_pid(pid).map_err(
            |err| Error::Io {
                message: err.to_string(),
            },
        )?);

        self.context.set_child(child).await;

        Ok(())
    }

    async fn start_simple(&mut self, ace: &Ace) -> Result<(), Error> {
        self.context
            .set_child(ace.run(&self.context.service.exec.start).await?)
            .await;

        if let Some(pid_file) = &self.context.service.service.pid_file {
            tokio::fs::write(pid_file, self.context.pid().await.unwrap().to_string())
                .await
                .ok();
        }
        Ok(())
    }

    async fn start_notify(&mut self, ace: &Ace, countdown: &Countdown) -> Result<(), Error> {
        let mut events = airupd().events.subscribe();
        let interests = [
            &self.context.service.name,
            &format!("{}.airs", self.context.service.name),
        ];
        let child = ace.run(&self.context.service.exec.start).await?;
        self.context.set_child(child).await;
        loop {
            let receive = match countdown.left() {
                Some(dur) => tokio::time::timeout(dur, events.recv()).await,
                None => Ok(events.recv().await),
            };
            let Ok(receive) = receive else {
                if let Some(child) = self.context.set_child(None).await {
                    child.kill().await.ok();
                }
                return Err(Error::TimedOut);
            };
            if let Ok(event) = receive {
                if event.id == "notify_active" && interests.contains(&&event.payload) {
                    break Ok(());
                }
            }
        }
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
