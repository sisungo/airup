use crate::process::{ExitStatus, Pid, Wait};
use ahash::AHashMap;
use std::{
    cmp::Ordering,
    convert::Infallible,
    os::unix::process::CommandExt,
    sync::{Mutex, OnceLock, RwLock},
};
use tokio::{
    process::{ChildStderr, ChildStdout},
    signal::unix::SignalKind,
    sync::mpsc,
};

static CHILD_QUEUE: OnceLock<ChildQueue> = OnceLock::new();

/// Waits for process termination in nonblocking mode.
fn wait_nonblocking(pid: Pid) -> std::io::Result<Option<Wait>> {
    let mut status = 0;
    let pid = unsafe { libc::waitpid(pid as _, &mut status, libc::WNOHANG) } as Pid;

    match pid.cmp(&0) {
        Ordering::Equal => Ok(None),
        Ordering::Less => Err(std::io::Error::last_os_error()),
        Ordering::Greater => Ok(Some(Wait::new(pid, ExitStatus::from_unix(status)))),
    }
}

/// Reloads the process image with the version on the filesystem.
pub fn reload_image() -> std::io::Result<Infallible> {
    Err(std::process::Command::new(std::env::current_exe()?)
        .args(std::env::args_os().skip(1))
        .exec())
}

/// Sends the given signal to the specified process.
pub async fn kill(pid: Pid, signum: i32) -> std::io::Result<()> {
    tokio::task::spawn_blocking(move || {
        let result = unsafe { libc::kill(pid as _, signum) };
        match result {
            0 => Ok(()),
            -1 => Err(std::io::Error::last_os_error()),
            _ => unreachable!(),
        }
    })
    .await
    .unwrap()
}

/// Representation of a running or exited child process.
#[derive(Debug)]
pub struct Child {
    pid: Pid,
    wait_queue: tokio::sync::Mutex<Option<mpsc::Receiver<Wait>>>,
    wait_cached: Mutex<Option<Wait>>,
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
}
impl Child {
    /// Returns OS-assign process ID of the child process.
    pub fn id(&self) -> Pid {
        self.pid
    }

    /// Converts from [std::process::Child] to [Child].
    pub fn from_std(c: std::process::Child) -> Self {
        // SAFETY: [std::process::Child] always represents to a valid child process.
        unsafe {
            Self::from_pid_unchecked(
                c.id() as _,
                c.stdout.and_then(|x| ChildStdout::from_std(x).ok()),
                c.stderr.and_then(|x| ChildStderr::from_std(x).ok()),
            )
        }
    }

    /// Creates a [Child] instance from PID. The PID must be a valid PID that belongs to child process of current process, or
    /// the behavior is undefined.
    ///
    /// ## Safety
    /// Current implementation of AirupFX process module doesn't cause safety issues when the PID doesn't meet the requirements,
    /// but the behavior may be changed in the future version.
    pub unsafe fn from_pid_unchecked(
        pid: Pid,
        stdout: Option<ChildStdout>,
        stderr: Option<ChildStderr>,
    ) -> Self {
        Self {
            pid,
            wait_queue: Some(child_queue().subscribe(pid)).into(),
            wait_cached: None.into(),
            stdout,
            stderr,
        }
    }

    /// Creates a [Child] instance from PID.
    ///
    /// ## Cancel Safety
    /// This method is cancel safe.
    pub async fn from_pid(pid: Pid) -> std::io::Result<Self> {
        match wait_nonblocking(pid)? {
            Some(wait) => Ok(Self {
                pid,
                wait_queue: None.into(),
                wait_cached: Some(wait).into(),
                stdout: None,
                stderr: None,
            }),
            None => Ok(unsafe { Self::from_pid_unchecked(pid, None, None) }),
        }
    }

    /// Waits until the process was terminated.
    ///
    /// ## Cancel Safety
    /// This method is cancel safe.
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

    /// Sends the specified signal to the child process.
    pub async fn kill(&self, sig: i32) -> std::io::Result<()> {
        let wait_cached = self.wait_cached.lock().unwrap().clone();
        match wait_cached {
            Some(_) => Err(std::io::ErrorKind::NotFound.into()),
            None => kill(self.pid, sig).await,
        }
    }

    pub fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.stdout.take()
    }

    pub fn take_stderr(&mut self) -> Option<ChildStderr> {
        self.stderr.take()
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
    /// Creates a new [ChildQueue] instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Initializes the global unique [ChildQueue] instance.
    ///
    /// ## Panic
    /// Panics if the instance is already set.
    pub fn init() {
        CHILD_QUEUE.set(Self::new()).unwrap();
        child_queue().start().ok();
    }

    /// Starts the child queue task.
    pub fn start(&'static self) -> anyhow::Result<()> {
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

    /// Creates a new [mpsc::Receiver] handle that will receive [Wait] sent after this call to `subscribe`.
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

    /// Sends the given [Wait] to the queue.
    pub async fn send(&self, wait: Wait) -> Option<()> {
        let entry = self.queue.write().unwrap().remove(&wait.pid());
        match entry {
            Some(x) => x.send(wait).await.ok().map(|_| ()),
            None => None,
        }
    }
}

/// Returns a reference to the global unique [ChildQueue] instance.
///
/// ## Panic
/// Panics if the instance has not been initialized yet.
fn child_queue() -> &'static ChildQueue {
    CHILD_QUEUE.get().unwrap()
}

/// An error occured by calling `wait` on a [Child].
#[derive(Debug, Clone, thiserror::Error)]
pub enum WaitError {
    #[error("subscribed queue for pid `{0}` was preempted")]
    PreemptedQueue(Pid),

    #[error("the child was already successfully waited without caching")]
    AlreadyWaited,
}

pub fn init() {
    ChildQueue::init();
}