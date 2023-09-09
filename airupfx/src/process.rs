//! A module for working with processes.

use crate::sys;
use std::{convert::Infallible, os::unix::process::CommandExt, sync::OnceLock};
use tokio::process::{ChildStderr, ChildStdout};

/// Represents to an OS-assigned process identifier.
pub type Pid = isize;

/// Returns the OS-assigned process identifier associated with this process.
pub fn id() -> Pid {
    static ID: OnceLock<Pid> = OnceLock::new();

    *ID.get_or_init(|| std::process::id() as _)
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
    sys::process::kill(pid, signum).await
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
pub struct Child(sys::process::Child);
impl Child {
    /// Returns OS-assign process ID of the child process.
    pub fn id(&self) -> Pid {
        self.0.id()
    }

    /// Converts from [std::process::Child] to [Child].
    pub fn from_std(c: std::process::Child) -> Self {
        Self(sys::process::Child::from_std(c))
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
        Self(sys::process::Child::from_pid_unchecked(pid, stdout, stderr))
    }

    /// Creates a [Child] instance from PID.
    ///
    /// ## Cancel Safety
    /// This method is cancel safe.
    pub async fn from_pid(pid: Pid) -> std::io::Result<Self> {
        Ok(Self(sys::process::Child::from_pid(pid).await?))
    }

    /// Waits until the process was terminated.
    ///
    /// ## Cancel Safety
    /// This method is cancel safe.
    pub async fn wait(&self) -> Result<Wait, WaitError> {
        self.0.wait().await.map_err(Into::into)
    }

    /// Sends the specified signal to the child process.
    pub async fn kill(&self, sig: i32) -> std::io::Result<()> {
        self.0.kill(sig).await
    }

    pub fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.0.take_stdout()
    }

    pub fn take_stderr(&mut self) -> Option<ChildStderr> {
        self.0.take_stderr()
    }
}

pub fn init() {
    sys::process::init();
}

pub async fn lock_handles() -> tokio::sync::RwLockReadGuard<'static, ()> {
    sys::process::lock_handles().await
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct WaitError(String);
impl From<sys::process::WaitError> for WaitError {
    fn from(value: sys::process::WaitError) -> Self {
        Self(value.to_string())
    }
}
