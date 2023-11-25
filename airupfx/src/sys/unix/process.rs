//! Process management on Unix platforms.
//!
//! This internally registers a `SIGCHLD` listener and spawns a background task to listen the signal. When registering
//! a new child process (e.g. by spawning), the PID is subscribed from the internal table. When a new sucessful call to
//! `waitpid()` completed, if the PID was previously subscribed, the result will be sent to the subscriber and then the
//! subscription is cancelled.

use super::std_port::CommandExt as _;
use crate::process::{ExitStatus, PiperHandle, Wait};
use ahash::AHashMap;
use std::{
    cmp,
    convert::Infallible,
    os::unix::process::CommandExt,
    sync::{Arc, Mutex, OnceLock, RwLock},
};
use sysinfo::UserExt;
use tokio::{signal::unix::SignalKind, sync::mpsc};

pub type Pid = libc::pid_t;

static CHILD_QUEUE: OnceLock<ChildQueue> = OnceLock::new();

/// Waits for process termination in nonblocking mode.
///
/// # Errors
/// An `Err(_)` is returned if the underlying OS function failed.
fn wait_nonblocking(pid: Pid) -> std::io::Result<Option<Wait>> {
    let mut status = 0;
    let pid = unsafe { libc::waitpid(pid as _, &mut status, libc::WNOHANG) } as Pid;

    match pid.cmp(&0) {
        cmp::Ordering::Equal => Ok(None),
        cmp::Ordering::Less => Err(std::io::Error::last_os_error()),
        cmp::Ordering::Greater => Ok(Some(Wait::new(pid, ExitStatus::from_unix(status)))),
    }
}

pub fn reload_image() -> std::io::Result<Infallible> {
    Err(std::process::Command::new(std::env::current_exe()?)
        .args(std::env::args_os().skip(1))
        .exec())
}

/// Sends the given signal to the specified process.
///
/// # Errors
/// An `Err(_)` is returned if the underlying OS function failed.
pub fn kill(pid: Pid, signum: i32) -> std::io::Result<()> {
    let result = unsafe { libc::kill(pid as _, signum) };
    match result {
        0 => Ok(()),
        -1 => Err(std::io::Error::last_os_error()),
        _ => unreachable!(),
    }
}

pub trait ExitStatusExt {
    /// Converts from a `status` returned by [`libc::waitpid`] to [`ExitStatus`].
    fn from_unix(status: libc::c_int) -> Self;
}
impl ExitStatusExt for crate::process::ExitStatus {
    fn from_unix(status: libc::c_int) -> Self {
        if libc::WIFEXITED(status) {
            Self::Exited(libc::WEXITSTATUS(status))
        } else if libc::WIFSIGNALED(status) {
            Self::Signaled(libc::WTERMSIG(status))
        } else {
            Self::Other
        }
    }
}

/// Representation of a running or exited child process.
#[derive(Debug)]
pub struct Child {
    pid: Pid,
    wait_queue: tokio::sync::Mutex<Option<mpsc::Receiver<Wait>>>,
    wait_cached: Mutex<Option<Wait>>,
    stdout: Option<Arc<PiperHandle>>,
    stderr: Option<Arc<PiperHandle>>,
}
impl Child {
    pub const fn id(&self) -> Pid {
        self.pid
    }

    fn from_std(c: std::process::Child) -> Self {
        let pid = c.id();
        let stdout = c
            .stdout
            .and_then(|x| tokio::process::ChildStdout::from_std(x).ok())
            .map(PiperHandle::new)
            .map(Arc::new);
        let stderr = c
            .stderr
            .and_then(|x| tokio::process::ChildStderr::from_std(x).ok())
            .map(PiperHandle::new)
            .map(Arc::new);
        Self {
            pid: pid as _,
            wait_queue: Some(child_queue().subscribe(pid as _)).into(),
            wait_cached: None.into(),
            stdout,
            stderr,
        }
    }

    /// Converts from process ID to [`Child`].
    ///
    /// # Safety
    /// The behavior converting from an invalid child PID to [`Child`] is not guaranteed.
    unsafe fn from_pid_unchecked(pid: Pid) -> Self {
        Self {
            pid,
            wait_queue: Some(child_queue().subscribe(pid)).into(),
            wait_cached: None.into(),
            stdout: None,
            stderr: None,
        }
    }

    pub fn from_pid(pid: Pid) -> std::io::Result<Self> {
        (wait_nonblocking(pid)?).map_or_else(
            || Ok(unsafe { Self::from_pid_unchecked(pid) }),
            |wait| {
                Ok(Self {
                    pid,
                    wait_queue: None.into(),
                    wait_cached: Some(wait).into(),
                    stdout: None,
                    stderr: None,
                })
            },
        )
    }

    pub async fn wait(&self) -> Result<Wait, WaitError> {
        let mut wait_queue = self.wait_queue.lock().await;

        if let Some(wait) = &*self.wait_cached.lock().unwrap() {
            return Ok(wait.clone());
        }

        let wait = wait_queue
            .as_mut()
            .ok_or(WaitError::AlreadyWaited)?
            .recv()
            .await
            .ok_or(WaitError::PreemptedQueue(self.pid))?;

        *self.wait_cached.lock().unwrap() = Some(wait.clone());
        *wait_queue = None;

        Ok(wait)
    }

    pub async fn send_signal(&self, sig: i32) -> std::io::Result<()> {
        let wait_cached = self.wait_cached.lock().unwrap().clone();
        match wait_cached {
            Some(_) => Err(std::io::ErrorKind::NotFound.into()),
            None => kill(self.pid, sig),
        }
    }

    pub async fn kill(&self) -> std::io::Result<()> {
        self.send_signal(super::signal::SIGKILL).await
    }

    pub fn stdout(&self) -> Option<Arc<PiperHandle>> {
        self.stdout.clone()
    }

    pub fn stderr(&self) -> Option<Arc<PiperHandle>> {
        self.stderr.clone()
    }
}
impl Drop for Child {
    fn drop(&mut self) {
        child_queue().unsubscribe(self.pid);
    }
}

/// A queue of waiting child processes.
#[derive(Debug, Default)]
struct ChildQueue {
    queue: RwLock<AHashMap<Pid, mpsc::Sender<Wait>>>,
}
impl ChildQueue {
    /// Creates a new [`ChildQueue`] instance.
    #[must_use]
    fn new() -> Self {
        Self::default()
    }

    /// Initializes the global unique [`ChildQueue`] instance.
    ///
    /// # Panics
    /// This method would panic if the instance is already set.
    pub fn init() {
        CHILD_QUEUE.set(Self::new()).unwrap();
        child_queue().start().ok();
    }

    /// Starts the child queue task.
    ///
    /// # Panics
    /// This method would panic if it is called more than once.
    fn start(&'static self) -> anyhow::Result<()> {
        static CALLED: OnceLock<()> = OnceLock::new();
        CALLED.set(()).unwrap();

        let mut signal = tokio::signal::unix::signal(SignalKind::child())?;
        tokio::spawn(async move {
            loop {
                signal.recv().await;
                loop {
                    let wait = match wait_nonblocking(-1) {
                        Ok(Some(x)) => x,
                        Ok(None) => break,
                        Err(x) => {
                            tracing::warn!("waitpid() failed: {}", x);
                            break;
                        }
                    };

                    if wait.code().is_some() || wait.signal().is_some() {
                        self.send(wait).await;
                        continue;
                    }
                }
            }
        });
        Ok(())
    }

    /// Creates a new [`mpsc::Receiver`] handle that will receive [`Wait`] sent after this call to `subscribe`.
    pub fn subscribe(&self, pid: Pid) -> mpsc::Receiver<Wait> {
        let mut lock = self.queue.write().unwrap();
        let (tx, rx) = mpsc::channel(1);
        lock.insert(pid, tx);
        rx
    }

    /// Removes a subscription ahead of time.
    pub fn unsubscribe(&self, pid: Pid) -> Option<()> {
        self.queue.write().unwrap().remove(&pid).map(|_| ())
    }

    /// Sends the given [`Wait`] to the queue.
    pub async fn send(&self, wait: Wait) -> Option<()> {
        let entry = self.queue.write().unwrap().remove(&wait.pid());
        match entry {
            Some(x) => x.send(wait).await.ok().map(|_| ()),
            None => None,
        }
    }
}

/// Returns a reference to the global unique [`ChildQueue`] instance.
///
/// # Panics
/// Panics if the instance has not been initialized yet.
fn child_queue() -> &'static ChildQueue {
    CHILD_QUEUE.get().unwrap()
}

/// Converts from [`crate::process::Command`] to [`std::process::Command`].
pub(crate) async fn command_to_std(
    command: &crate::process::Command,
) -> anyhow::Result<std::process::Command> {
    let mut result = std::process::Command::new(&command.program);
    command.args.iter().for_each(|x| {
        result.arg(x);
    });
    if let Some(x) = &command.arg0 {
        result.arg0(x);
    }
    if let Some(x) = command.env.uid {
        result.uid(x);
    }
    if let Some(x) = command.env.gid {
        result.gid(x);
    }
    if let Some(x) = &command.env.working_dir {
        result.current_dir(x);
    }
    if command.env.clear_vars {
        result.env_clear();
    }
    command.env.vars.iter().for_each(|(k, v)| match v {
        Some(v) => {
            result.env(k, v);
        }
        None => {
            result.env_remove(k);
        }
    });
    result
        .stdout(command.env.stdout.to_std().await?)
        .stderr(command.env.stderr.to_std().await?);
    if command.env.setsid {
        result.setsid();
    }

    Ok(result)
}

pub(crate) fn command_login(
    env: &mut crate::process::CommandEnv,
    name: &str,
) -> anyhow::Result<()> {
    let (uid, gid) = crate::env::with_user_by_name(name, |entry| (**entry.id(), *entry.group_id()))
        .ok_or_else(|| anyhow::anyhow!("user \"{name}\" not found"))?;
    env.uid(uid).gid(gid);

    Ok(())
}

pub async fn spawn(cmd: &crate::process::Command) -> anyhow::Result<Child> {
    Ok(Child::from_std(command_to_std(cmd).await?.spawn()?))
}

/// An error occured by calling `wait` on a [`Child`].
#[derive(Debug, Clone, thiserror::Error)]
pub enum WaitError {
    #[error("subscribed queue for pid `{0}` was preempted")]
    PreemptedQueue(Pid),

    #[error("the child was already successfully waited without caching")]
    AlreadyWaited,
}

/// Initializes the process manager.
pub fn init() {
    ChildQueue::init();
}
