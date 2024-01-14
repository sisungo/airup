//! Process management on Unix platforms.
//!
//! This internally registers a `SIGCHLD` listener and spawns a background task to listen the signal. When registering
//! a new child process (e.g. by spawning), the PID is subscribed from the internal table. When a new sucessful call to
//! `waitpid()` completed, if the PID was previously subscribed, the result will be sent to the subscriber and then the
//! subscription is cancelled.

#![allow(unstable_name_collisions)]

use super::std_port::CommandExt as _;
use crate::{
    io::PiperHandle,
    process::{ExitStatus, Wait},
};
use ahash::AHashMap;
use std::{
    cmp,
    convert::Infallible,
    os::unix::process::CommandExt as _,
    sync::{Arc, OnceLock, RwLock},
    time::Duration,
};
use tokio::{signal::unix::SignalKind, sync::watch};

pub type Pid = libc::pid_t;

static CHILD_QUEUE: OnceLock<ChildQueue> = OnceLock::new();

pub fn reload_image() -> std::io::Result<Infallible> {
    Err(std::process::Command::new(std::env::current_exe()?)
        .args(std::env::args_os().skip(1))
        .exec())
}

pub fn is_forking_supervisable() -> bool {
    std::process::id() == 1
}

/// Sends the given signal to the specified process.
///
/// # Errors
/// An `Err(_)` is returned if the underlying OS function failed.
fn kill(pid: Pid, signum: i32) -> std::io::Result<()> {
    let result = unsafe { libc::kill(pid as _, signum) };
    match result {
        0 => Ok(()),
        _ => Err(std::io::Error::last_os_error()),
    }
}

/// Sends a signal to all running processes, then wait for them to be terminated. If the timeout expired, the processes are
/// force-killed.
pub(crate) async fn kill_all(timeout: Duration) {
    eprintln!("Sending SIGTERM to all processes");
    kill(-1, super::signal::SIGTERM).ok();

    eprintln!("Waiting for all processes to be terminated");
    let _lock = child_queue().lock.lock().await;
    tokio::time::timeout(
        timeout,
        tokio::task::spawn_blocking(|| {
            let mut status = 0;
            while unsafe { libc::wait(&mut status) > 0 } {}
        }),
    )
    .await
    .ok();
    drop(_lock);

    eprintln!("Sending SIGKILL to all processes");
    kill(-1, super::signal::SIGKILL).ok();
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
pub(crate) struct Child {
    pid: Pid,
    wait_queue: watch::Receiver<Option<Wait>>,
    stdout: Option<Arc<PiperHandle>>,
    stderr: Option<Arc<PiperHandle>>,
}
impl Child {
    pub(crate) const fn id(&self) -> Pid {
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
            wait_queue: child_queue().subscribe(pid as _),
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
            wait_queue: child_queue().subscribe(pid),
            stdout: None,
            stderr: None,
        }
    }

    pub(crate) fn from_pid(pid: Pid) -> std::io::Result<Self> {
        (wait_nonblocking(pid)?).map_or_else(
            || Ok(unsafe { Self::from_pid_unchecked(pid) }),
            |_| Err(std::io::ErrorKind::NotFound.into()),
        )
    }

    pub(crate) async fn wait(&self) -> Result<Wait, WaitError> {
        let mut wait_queue = self.wait_queue.clone();
        let wait = wait_queue.wait_for(|x| x.is_some()).await.unwrap();
        Ok(wait.clone().unwrap())
    }

    pub(crate) fn send_signal(&self, sig: i32) -> std::io::Result<()> {
        if self.wait_queue.borrow().is_none() {
            kill(self.pid, sig)
        } else {
            Err(std::io::ErrorKind::NotFound.into())
        }
    }

    pub(crate) fn kill(&self) -> std::io::Result<()> {
        self.send_signal(super::signal::SIGKILL)
    }

    pub(crate) fn stdout(&self) -> Option<Arc<PiperHandle>> {
        self.stdout.clone()
    }

    pub(crate) fn stderr(&self) -> Option<Arc<PiperHandle>> {
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
    queue: RwLock<AHashMap<Pid, watch::Sender<Option<Wait>>>>,
    lock: tokio::sync::Mutex<()>,
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
    fn init() {
        CHILD_QUEUE.set(Self::new()).unwrap();
        child_queue().start().ok();
    }

    /// Starts the child queue task.
    fn start(&'static self) -> anyhow::Result<tokio::task::JoinHandle<()>> {
        let mut signal = tokio::signal::unix::signal(SignalKind::child())?;
        Ok(tokio::spawn(async move {
            loop {
                signal.recv().await;
                loop {
                    let _lock = self.lock.lock().await;
                    let wait = match wait_nonblocking(-1) {
                        Ok(Some(x)) => x,
                        Ok(None) => break,
                        Err(x) => {
                            tracing::warn!("waitpid() failed: {}", x);
                            break;
                        }
                    };
                    drop(_lock);

                    if wait.code().is_some() || wait.signal().is_some() {
                        self.send(wait).await;
                        continue;
                    }
                }
            }
        }))
    }

    /// Creates a new [`mpsc::Receiver`] handle that will receive [`Wait`] sent after this call to `subscribe`.
    fn subscribe(&self, pid: Pid) -> watch::Receiver<Option<Wait>> {
        let mut lock = self.queue.write().unwrap();
        let (tx, rx) = watch::channel(None);
        lock.insert(pid, tx);
        rx
    }

    /// Removes a subscription ahead of time.
    fn unsubscribe(&self, pid: Pid) -> Option<()> {
        self.queue.write().unwrap().remove(&pid).map(|_| ())
    }

    /// Sends the given [`Wait`] to the queue.
    async fn send(&self, wait: Wait) -> Option<()> {
        let entry = self.queue.write().unwrap().remove(&(wait.pid() as _));
        match entry {
            Some(x) => x.send(Some(wait)).ok().map(|_| ()),
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
        cmp::Ordering::Greater => Ok(Some(Wait::new(pid as _, ExitStatus::from_unix(status)))),
    }
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
    if let Some(x) = &command.env.groups {
        result.groups(x);
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
    let (uid, gid, groups_id) = crate::env::with_user_by_name(name, |entry| {
        (
            **entry.id(),
            *entry.group_id(),
            entry
                .groups()
                .into_iter()
                .map(|x| **x.id())
                .collect::<Vec<_>>(),
        )
    })
    .ok_or_else(|| anyhow::anyhow!("user \"{name}\" not found"))?;
    env.uid(uid).gid(gid).groups(groups_id);

    Ok(())
}

pub(crate) async fn spawn(cmd: &crate::process::Command) -> anyhow::Result<Child> {
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
