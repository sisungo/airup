//! A module for working with processes.

use ahash::AHashMap;
use std::{
    cmp::Ordering,
    convert::Infallible,
    future::Future,
    os::unix::process::CommandExt,
    sync::{OnceLock, RwLock, Mutex},
};
use tokio::{
    process::{ChildStderr, ChildStdout},
    signal::unix::SignalKind,
    sync::mpsc,
};

static CHILD_QUEUE: OnceLock<ChildQueue> = OnceLock::new();

/// Represents to an OS-assigned process identifier.
pub type Pid = i64;

/// Terminal line hangup.
pub const SIGHUP: i32 = libc::SIGHUP;

/// Interrupt program.
pub const SIGINT: i32 = libc::SIGINT;

/// Quit program.
pub const SIGQUIT: i32 = libc::SIGQUIT;

/// Write on a pipe with no reader.
pub const SIGPIPE: i32 = libc::SIGPIPE;

/// Software termination signal.
pub const SIGTERM: i32 = libc::SIGTERM;

/// Stop signal generated from keyboard.
pub const SIGTSTP: i32 = libc::SIGTSTP;

/// Child status has changed.
pub const SIGCHLD: i32 = libc::SIGCHLD;

/// Background read attempted from control terminal.
pub const SIGTTIN: i32 = libc::SIGTTIN;

/// Background write attempted to control terminal.
pub const SIGTTOU: i32 = libc::SIGTTOU;

/// I/O is possible on a descriptor (see fcntl(2)).
pub const SIGIO: i32 = libc::SIGIO;

/// User defined signal 1.
pub const SIGUSR1: i32 = libc::SIGUSR1;

/// User defined signal 2.
pub const SIGUSR2: i32 = libc::SIGUSR2;

/// Window size change
pub const SIGWINCH: i32 = libc::SIGWINCH;

/// Registers a signal handler.
pub fn signal<
    F: FnMut(i32) -> T + Send + Sync + 'static,
    T: Future<Output = ()> + Send + 'static,
>(
    signum: i32,
    mut op: F,
) -> anyhow::Result<()> {
    let mut signal = tokio::signal::unix::signal(SignalKind::from_raw(signum))?;
    tokio::spawn(async move {
        loop {
            signal.recv().await;
            op(signum).await;
        }
    });

    Ok(())
}

/// Ignores a signal.
pub fn ignore(signum: i32) -> anyhow::Result<()> {
    signal(signum, |_| async {})
}

/// Ignores all signals in the list. Any errors will be ignored.
pub fn ignore_all<I: IntoIterator<Item = i32>>(signum_list: I) {
    signum_list.into_iter().for_each(|signum| {
        ignore(signum).ok();
    });
}

/// Returns the OS-assigned process identifier associated with this process.
pub fn id() -> Pid {
    static ID: OnceLock<Pid> = OnceLock::new();

    *ID.get_or_init(|| std::process::id() as _)
}

/// Waits for process termination in nonblocking mode.
pub fn wait_nonblocking(pid: Pid) -> std::io::Result<Option<Wait>> {
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

/// Called when a fatal error occured.
///
/// If the process has `pid==1`, this will start a shell and reloads the process image. Otherwise this will make current
/// process exit.
pub fn emergency() -> ! {
    tracing::error!(target: "console", "A fatal error occured.");
    if id() == 1 {
        loop {
            tracing::error!(target: "console", "Launching shell...");
            if let Err(e) = launch_shell() {
                tracing::error!(target: "console", "failed to start shell: {e}");
            }

            tracing::error!(target: "console", "Reloading `airupd` process image...");
            if let Err(e) = reload_image() {
                tracing::error!(target: "console", "failed to reload `airupd` image: {e}");
            }
        }
    } else {
        std::process::exit(1);
    }
}

/// Opens a shell and waits for it to exit.
fn launch_shell() -> std::io::Result<()> {
    std::process::Command::new("sh").spawn()?.wait().map(|_| ())
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

/// Describes the result of calling `wait`-series methods.
#[derive(Debug, Clone)]
pub struct Wait {
    pid: Pid,
    pub exit_status: ExitStatus,
}
impl Wait {
    /// Creates a new [Wait] object.
    pub fn new(pid: Pid, exit_status: ExitStatus) -> Self {
        Self { pid, exit_status }
    }

    /// Returns the OS-assigned process identifier associated with the wait result.
    pub fn pid(&self) -> Pid {
        self.pid
    }

    /// Returns `true` if the process terminated normally by a call to `exit`.
    pub fn code(&self) -> Option<libc::c_int> {
        match self.exit_status {
            ExitStatus::Exited(x) => Some(x),
            _ => None,
        }
    }

    /// Returns `true` if the process was terminated by a signal.
    pub fn signal(&self) -> Option<libc::c_int> {
        match self.exit_status {
            ExitStatus::Signaled(x) => Some(x),
            _ => None,
        }
    }

    /// Returns `true` if the process was successfully completed.
    pub fn is_success(&self) -> bool {
        self.exit_status.is_success()
    }
}

/// Represents to an exit status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExitStatus {
    /// The process was terminated normally by a call to [libc::_exit] or [libc::exit].
    Exited(libc::c_int),

    /// The process was terminated due to receipt of a signal.
    Signaled(libc::c_int),
}
impl ExitStatus {
    /// Represents to a successful exit.
    pub const SUCCESS: Self = Self::Exited(0);

    /// Converts from a `status` returned by [libc::waitpid] to [ExitStatus].
    pub fn from_unix(status: libc::c_int) -> Self {
        if libc::WIFEXITED(status) {
            Self::Exited(libc::WEXITSTATUS(status))
        } else if libc::WIFSIGNALED(status) {
            Self::Signaled(libc::WTERMSIG(status))
        } else {
            unreachable!()
        }
    }

    /// Returns `true` if the process was successfully completed.
    pub fn is_success(&self) -> bool {
        *self == Self::SUCCESS
    }
}

/// Representation of a running or exited child process.
#[derive(Debug)]
pub struct Child {
    pid: Pid,
    wait_queue: tokio::sync::Mutex<Option<mpsc::Receiver<Wait>>>,
    wait_cached: Mutex<Option<Wait>>,
    pub stdout: Option<ChildStdout>,
    pub stderr: Option<ChildStderr>,
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
                c.stdout.map(|x| ChildStdout::from_std(x).ok()).flatten(),
                c.stderr.map(|x| ChildStderr::from_std(x).ok()).flatten(),
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
    pub async fn from_pid(pid: Pid) -> std::io::Result<Self> {
        let _lock = child_queue().lock_waiter().await;
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
        if let Some(wait) = &*self.wait_cached.lock().unwrap() {
            return Ok(wait.clone());
        }

        let mut wait_queue = self.wait_queue.lock().await;

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
}
impl Drop for Child {
    fn drop(&mut self) {
        child_queue().unsubscribe(self.pid);
    }
}

/// A queue of waiting child processes.
#[derive(Debug, Default)]
pub struct ChildQueue {
    waiter_lock: tokio::sync::RwLock<()>,
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
                    let _lock = self.waiter_lock.write().await;
                    let wait = match wait_nonblocking(-1) {
                        Ok(Some(x)) => x,
                        Ok(None) => continue,
                        Err(x) => {
                            tracing::warn!("waitpid() failed: {}", x);
                            break;
                        }
                    };

                    if wait.code().is_some() || wait.signal().is_some() {
                        self.send(wait).await;
                        break;
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

    /// Gets a shared lock access of the waiter lock.
    pub async fn lock_waiter(&self) -> tokio::sync::RwLockReadGuard<'_, ()> {
        self.waiter_lock.read().await
    }
}

/// Returns a reference to the global unique [ChildQueue] instance.
///
/// ## Panic
/// Panics if the instance has not been initialized yet.
pub fn child_queue() -> &'static ChildQueue {
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
