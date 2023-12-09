//! Tasks of the Airup supervisor.

mod cleanup_service;
pub mod feedback;
mod reload_service;
mod start_service;
mod stop_service;

pub use cleanup_service::{cleanup_service, CleanupServiceHandle};
pub use feedback::TaskFeedback;
pub use reload_service::ReloadServiceHandle;
pub use start_service::StartServiceHandle;
pub use stop_service::StopServiceHandle;
use tokio_util::sync::CancellationToken;

use super::SupervisorContext;
use airup_sdk::Error;
use airupfx::prelude::*;
use std::{future::Future, path::PathBuf};
use tokio::sync::watch;

/// Representation of handle to a task.
pub trait TaskHandle: Send + Sync + 'static {
    /// Returns type name of the task.
    fn task_type(&self) -> &'static str;

    /// Sends an interruption request to the task.
    ///
    /// **NOTE**: It's determined by the task logic that what time it can be interrupted and the method immediately returns. To
    /// wait until the task terminated, please call [`TaskHandle::wait`].
    fn send_interrupt(&self);

    /// Waits for completion of the task.
    ///
    /// If the task has been interrupted, this should return `Err(Error::TaskInterrupted)`. Otherwise it waits until the task
    /// has completed.
    ///
    /// # Cancel Safety
    /// This method is cancel-safe.
    fn wait(&self) -> BoxFuture<Result<TaskFeedback, Error>>;
}

/// An [`TaskHandle`] implementation that immediately successfully completes.
pub struct EmptyTaskHandle;
impl TaskHandle for EmptyTaskHandle {
    fn task_type(&self) -> &'static str {
        "Empty"
    }
    fn send_interrupt(&self) {}
    fn wait(&self) -> BoxFuture<Result<TaskFeedback, Error>> {
        Box::pin(async { Ok(().into()) })
    }
}

/// A helper type for implementing [`TaskHandle`].
#[derive(Debug)]
pub struct TaskHelperHandle {
    cancellation_token: CancellationToken,
    done: watch::Receiver<Option<Result<TaskFeedback, Error>>>,
}
impl TaskHandle for TaskHelperHandle {
    fn task_type(&self) -> &'static str {
        panic!("TaskHelperHandle::task_type should NOT be called")
    }

    fn send_interrupt(&self) {
        self.cancellation_token.cancel();
    }

    fn wait(&self) -> BoxFuture<Result<TaskFeedback, Error>> {
        Box::pin(async {
            let mut receiver = self.done.clone();
            let x = receiver
                .wait_for(|x| x.is_some())
                .await
                .map_err(|_| Error::TaskInterrupted)?;
            x.as_ref().unwrap().clone() // `x` is guaranteed `Some(_)` here
        })
    }
}

/// A helper type for implementing [`TaskHandle`], which acts as backend type of [`TaskHelperHandle`].
#[derive(Debug)]
pub struct TaskHelper {
    cancellation_token: CancellationToken,
    done: watch::Sender<Option<Result<TaskFeedback, Error>>>,
}
impl TaskHelper {
    /// Executes a [`Future`] in an interruptable scope. If this task is interrupted, returns `Err(Error::TaskInterrupted)`,
    /// otherwise returns `Ok(_)`.
    pub async fn would_interrupt<T>(&self, future: impl Future<Output = T>) -> Result<T, Error> {
        tokio::select! {
            val = future => Ok(val),
            _ = self.cancellation_token.cancelled() => Err(Error::TaskInterrupted),
        }
    }

    /// If this task is interrupted, returns `Err(Error::TaskInterrupted)`, otherwise returns `Ok(_)`.
    pub fn interruptable_point(&self) -> Result<(), Error> {
        match self.cancellation_token.is_cancelled() {
            true => Err(Error::TaskInterrupted),
            false => Ok(()),
        }
    }

    /// Mark the task done and returns a value.
    pub fn finish<T: Into<TaskFeedback>>(&self, val: Result<T, Error>) {
        self.done.send(Some(val.map(|x| x.into()))).ok();
    }
}

/// Returns a pair of [`TaskHelper`] and [`TaskHelperHandle`].
pub fn task_helper() -> (TaskHelperHandle, TaskHelper) {
    let cancellation_token = CancellationToken::new();
    let (done_tx, done_rx) = watch::channel(None);

    let helper = TaskHelper {
        cancellation_token: cancellation_token.clone(),
        done: done_tx,
    };
    let handle = TaskHelperHandle {
        cancellation_token,
        done: done_rx,
    };

    (handle, helper)
}

/// Creates an [`Ace`] instance matching the given [`SupervisorContext`].
pub async fn ace(context: &SupervisorContext) -> Result<Ace, Error> {
    let mut ace = Ace::new();

    async fn env_convert(
        env: &airup_sdk::files::service::Env,
    ) -> anyhow::Result<airupfx::process::CommandEnv> {
        let mut result = airupfx::process::CommandEnv::new();

        let to_ace = |x| match x {
            airup_sdk::files::service::Stdio::Inherit => airupfx::process::Stdio::Inherit,
            airup_sdk::files::service::Stdio::File(path) => airupfx::process::Stdio::File(path),
            airup_sdk::files::service::Stdio::Log => airupfx::process::Stdio::Piped,
        };

        result
            .login(env.user.as_deref())?
            .uid(env.uid)
            .gid(env.gid)
            .stdout(to_ace(env.stdout.clone()))
            .stderr(to_ace(env.stderr.clone()))
            .clear_vars(env.clear_vars)
            .vars::<_, String, _, String>(env.vars.clone().into_iter())
            .working_dir::<PathBuf, _>(env.working_dir.clone())
            .setsid(true);

        Ok(result)
    }

    ace.env = env_convert(&context.service.env)
        .await
        .map_err(|x| Error::Io {
            message: x.to_string(),
        })?;
    ace.env.var("AIRUP_SERVICE", context.service.name.clone());
    if let Some(pid) = context.pid().await {
        ace.env.var("AIRUP_SERVICE_MAIN_PID", pid.to_string());
    }

    Ok(ace)
}
