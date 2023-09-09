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

use super::SupervisorContext;
use airupfx::prelude::*;
use airup_sdk::Error;
use std::future::Future;
use tokio::sync::watch;

/// Representation of handle to a task.
pub trait TaskHandle: Send + Sync + 'static {
    /// Returns type name of the task.
    fn task_type(&self) -> &'static str;

    /// Sends an interruption request to the task.
    fn send_interrupt(&self);

    /// Waits for completion of the task.
    ///
    /// If the task has been interrupted, this should return `Err(Error::TaskInterrupted)`. Otherwise it waits until the task
    /// has completed.
    ///
    /// ## Cancel Safety
    /// This method is cancel-safe.
    fn wait(&self) -> BoxFuture<Result<TaskFeedback, Error>>;
}

/// A helper type for implementing [TaskHandle].
#[derive(Debug)]
pub struct TaskHelperHandle {
    int_flag: watch::Sender<bool>,
    done: watch::Receiver<Option<Result<TaskFeedback, Error>>>,
}
impl TaskHandle for TaskHelperHandle {
    fn task_type(&self) -> &'static str {
        panic!("TaskHelperHandle::task_type should NOT be called")
    }

    fn send_interrupt(&self) {
        let _ = self.int_flag.send(true);
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

/// A helper type for implementing [TaskHandle], backend type of [TaskHelperHandle].
#[derive(Debug)]
pub struct TaskHelper {
    int_flag: watch::Receiver<bool>,
    done: watch::Sender<Option<Result<TaskFeedback, Error>>>,
}
impl TaskHelper {
    /// Executes a [Future] in an interruptable scope. If this task is interrupted, returns `Err(Error::TaskInterrupted)`,
    /// otherwise returns `Ok(_)`.
    pub async fn interruptable_scope<T, F: Future<Output = T>>(
        &self,
        future: F,
    ) -> Result<T, Error> {
        let mut int_flag = self.int_flag.clone();
        tokio::select! {
            val = future => Ok(val),
            Ok(_) = int_flag.wait_for(|x| *x) => Err(Error::TaskInterrupted),
        }
    }

    /// If this task is interrupted, returns `Err(Error::TaskInterrupted)`, otherwise returns `Ok(_)`.
    pub fn interruptable_point(&self) -> Result<(), Error> {
        match *self.int_flag.borrow() {
            true => Err(Error::TaskInterrupted),
            false => Ok(()),
        }
    }

    /// Mark the task done and returns a value.
    pub fn finish<T: Into<TaskFeedback>>(&self, val: Result<T, Error>) {
        self.done.send(Some(val.map(|x| x.into()))).ok();
    }
}

/// Returns a pair of [TaskHelper] and [TaskHelperHandle].
pub fn task_helper() -> (TaskHelperHandle, TaskHelper) {
    let (int_flag_tx, int_flag_rx) = watch::channel(false);
    let (done_tx, done_rx) = watch::channel(None);

    let helper = TaskHelper {
        int_flag: int_flag_rx,
        done: done_tx,
    };
    let handle = TaskHelperHandle {
        int_flag: int_flag_tx,
        done: done_rx,
    };

    (handle, helper)
}

/// Creates an [Ace] instance matching the given [SupervisorContext].
pub async fn ace(context: &SupervisorContext) -> Result<Ace, Error> {
    let mut ace = Ace::new();

    ace.env = context.service.env.into_ace()?;
    if let Some(pid) = context.pid().await {
        ace.env.var("MAINPID", pid.to_string());
    }

    Ok(ace)
}
