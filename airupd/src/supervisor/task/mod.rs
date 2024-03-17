//! Tasks of the Airup supervisor.

pub mod cleanup;
pub mod health_check;
pub mod reload;
pub mod start;
pub mod stop;

use super::SupervisorContext;
use airup_sdk::Error;
use airupfx::{io::line_piper::Callback as LinePiperCallback, prelude::*};
use std::{future::Future, path::PathBuf, pin::Pin};
use tokio::sync::watch;

/// Representation of handle to a task.
pub trait TaskHandle: Send + Sync + 'static {
    /// Returns class of the task.
    fn task_class(&self) -> &'static str;

    /// Returns `true` if the task is important.
    fn is_important(&self) -> bool;

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

macro_rules! task_feedback_from {
    ($t:ty, $v:tt) => {
        impl From<$t> for TaskFeedback {
            fn from(val: $t) -> Self {
                Self::$v(val)
            }
        }
    };
}

#[derive(Debug, Clone)]
pub enum TaskFeedback {
    Nothing(()),
}
task_feedback_from!((), Nothing);

/// A helper type for implementing [`TaskHandle`].
#[derive(Debug)]
pub struct TaskHelperHandle {
    interrupt_flag: watch::Sender<bool>,
    done: watch::Receiver<Option<Result<TaskFeedback, Error>>>,
}
impl TaskHandle for TaskHelperHandle {
    fn task_class(&self) -> &'static str {
        panic!("TaskHelperHandle::task_class should be never called")
    }

    fn is_important(&self) -> bool {
        panic!("TaskHelperHandle::is_important should be never called")
    }

    fn send_interrupt(&self) {
        self.interrupt_flag.send(true).ok();
    }

    fn wait(&self) -> BoxFuture<Result<TaskFeedback, Error>> {
        Box::pin(async {
            let mut receiver = self.done.clone();
            let x = receiver
                .wait_for(|x| x.is_some())
                .await
                .map_err(|_| Error::TaskInterrupted)?;

            x.as_ref()
                .expect("`watch::Receiver::wait_for` should only return expected value")
                .clone()
        })
    }
}

/// A helper type for implementing [`TaskHandle`], which acts as backend type of [`TaskHelperHandle`].
#[derive(Debug)]
pub struct TaskHelper {
    interrupt_flag: watch::Receiver<bool>,
    done: watch::Sender<Option<Result<TaskFeedback, Error>>>,
}
impl TaskHelper {
    /// Executes a [`Future`] in an interruptable scope. If this task is interrupted, returns `Err(Error::TaskInterrupted)`,
    /// otherwise returns `Ok(_)`.
    pub async fn would_interrupt<T>(&self, future: impl Future<Output = T>) -> Result<T, Error> {
        let mut rx = self.interrupt_flag.clone();
        tokio::select! {
            val = future => Ok(val),
            _ = rx.wait_for(|x| *x) => Err(Error::TaskInterrupted),
        }
    }

    /// If this task is interrupted, returns `Err(Error::TaskInterrupted)`, otherwise returns `Ok(_)`.
    pub fn interruptable_point(&self) -> Result<(), Error> {
        match *self.interrupt_flag.borrow() {
            true => Err(Error::TaskInterrupted),
            false => Ok(()),
        }
    }

    /// Mark the task done and returns a value.
    pub fn finish<T: Into<TaskFeedback>>(&self, val: Result<T, Error>) {
        self.done.send(Some(val.map(|x| x.into()))).ok();
    }
}

pub struct Empty;
impl TaskHandle for Empty {
    fn task_class(&self) -> &'static str {
        "Empty"
    }

    fn is_important(&self) -> bool {
        false
    }

    fn send_interrupt(&self) {}

    fn wait(&self) -> BoxFuture<Result<TaskFeedback, Error>> {
        Box::pin(async { Ok(TaskFeedback::Nothing(())) })
    }
}

/// Returns a pair of [`TaskHelper`] and [`TaskHelperHandle`].
pub fn task_helper() -> (TaskHelperHandle, TaskHelper) {
    let (tx, rx) = watch::channel(false);
    let (done_tx, done_rx) = watch::channel(None);

    let helper = TaskHelper {
        interrupt_flag: rx,
        done: done_tx,
    };
    let handle = TaskHelperHandle {
        interrupt_flag: tx,
        done: done_rx,
    };

    (handle, helper)
}

#[derive(Clone)]
struct LogCallback {
    name: String,
    module: &'static str,
}
impl LogCallback {
    pub fn new(name: String, module: &'static str) -> Self {
        Self { name, module }
    }
}
impl LinePiperCallback for LogCallback {
    fn invoke<'a>(
        &'a self,
        msg: &'a [u8],
    ) -> Pin<Box<dyn for<'b> Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            crate::app::airupd()
                .logger
                .write(&self.name, self.module, msg)
                .await
                .ok();
        })
    }
    fn clone_boxed(&self) -> Box<dyn LinePiperCallback> {
        Box::new(self.clone())
    }
}

async fn ace_environment(
    service: &airup_sdk::files::Service,
) -> anyhow::Result<airupfx::process::CommandEnv> {
    let env = &service.env;
    let mut result = airupfx::process::CommandEnv::new();
    let log = |y| {
        let module = match y {
            1 => "stdout",
            2 => "stderr",
            _ => unreachable!(),
        };

        let callback = LogCallback::new(format!("airup_service_{}", service.name), module);
        airupfx::process::Stdio::Callback(Box::new(callback))
    };

    let to_ace = |x, y| match x {
        airup_sdk::files::service::Stdio::Nulldev => airupfx::process::Stdio::Nulldev,
        airup_sdk::files::service::Stdio::Inherit => airupfx::process::Stdio::Inherit,
        airup_sdk::files::service::Stdio::File(path) => airupfx::process::Stdio::File(path),
        airup_sdk::files::service::Stdio::Log => log(y),
    };

    result
        .login(env.login.as_deref())?
        .uid(env.uid)
        .gid(env.gid)
        .stdin(to_ace(env.stdin.clone(), 0))
        .stdout(to_ace(env.stdout.clone(), 1))
        .stderr(to_ace(env.stderr.clone(), 2))
        .clear_vars(env.clear_vars)
        .vars::<String, _, String>(env.vars.clone().into_iter())
        .working_dir::<PathBuf, _>(env.working_dir.clone())
        .setsid(true);

    Ok(result)
}

/// Creates an [`Ace`] instance matching the given [`SupervisorContext`].
pub(in crate::supervisor) async fn ace(context: &SupervisorContext) -> Result<Ace, Error> {
    let mut ace = Ace::new();

    ace.realm.clone_from(&context.realm);
    ace.env = ace_environment(&context.service)
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
