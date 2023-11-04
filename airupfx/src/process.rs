//! A module for working with processes.

use crate::sys;
use once_cell::sync::Lazy;
use std::{
    convert::Infallible,
    ffi::OsString,
    ops::{Deref, DerefMut},
    path::PathBuf,
};
use tokio::process::{ChildStderr, ChildStdout};

/// Represents to an OS-assigned process identifier.
pub type Pid = libc::pid_t;

/// The OS-assigned process identifier associated with this process.
pub static ID: Lazy<u32> = Lazy::new(std::process::id);

/// Reloads the process image with the version on the filesystem.
///
/// ## Errors
/// An `Err(_)` is returned if the underlying OS function failed.
pub fn reload_image() -> std::io::Result<Infallible> {
    sys::process::reload_image()
}

/// Called when a fatal error has occured.
///
/// If the process has `pid==1`, this will start a shell and reloads the process image. Otherwise this will make current
/// process exit.
pub fn emergency() -> ! {
    if *ID == 1 {
        loop {
            tracing::error!(target: "console", "A fatal error has occured. Starting shell...");
            if let Err(e) = shell() {
                tracing::error!(target: "console", "failed to start shell: {e}");
            }

            tracing::error!(target: "console", "Rebooting the userspace...");
            if let Err(e) = reload_image() {
                tracing::error!(target: "console", "Failed to reboot the userspace: {e}");
            }
        }
    } else {
        tracing::error!(target: "console", "A fatal error has occured. Exiting...");
        std::process::exit(1);
    }
}

/// Opens a shell and waits for it to exit.
fn shell() -> std::io::Result<()> {
    std::process::Command::new("sh").spawn()?.wait().map(|_| ())
}

/// Sends the given signal to the specified process.
///
/// ## Errors
/// An `Err(_)` is returned if the underlying OS function failed.
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
    /// The process was terminated normally by a call to [`libc::_exit`] or [`libc::exit`].
    Exited(libc::c_int),

    /// The process was terminated due to receipt of a signal.
    Signaled(libc::c_int),

    /// The process was not terminated.
    Other,
}
impl ExitStatus {
    /// Represents to a successful exit.
    pub const SUCCESS: Self = Self::Exited(0);

    /// Converts from a `status` returned by [`libc::waitpid`] to [`ExitStatus`].
    pub fn from_unix(status: libc::c_int) -> Self {
        if libc::WIFEXITED(status) {
            Self::Exited(libc::WEXITSTATUS(status))
        } else if libc::WIFSIGNALED(status) {
            Self::Signaled(libc::WTERMSIG(status))
        } else {
            Self::Other
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

    /// Converts from [`std::process::Child`] to [`Child`].
    pub fn from_std(c: std::process::Child) -> Self {
        Self(sys::process::Child::from_std(c))
    }

    /// Creates a [`Child`] instance from PID. The PID must be a valid PID that belongs to child process of current process, or
    /// the behavior is undefined.
    ///
    /// # Safety
    /// Current implementation of `AirupFX` process module doesn't cause safety issues when the PID doesn't meet the requirements,
    /// but the behavior may be changed in the future version.
    pub unsafe fn from_pid_unchecked(
        pid: Pid,
        stdout: Option<ChildStdout>,
        stderr: Option<ChildStderr>,
    ) -> Self {
        Self(sys::process::Child::from_pid_unchecked(pid, stdout, stderr))
    }

    /// Creates a [`Child`] instance from PID.
    ///
    /// ## Cancel Safety
    /// This method is cancel safe.
    ///
    /// ## Errors
    /// An `Err(_)` is returned if the process is not a valid child process of current process.
    pub fn from_pid(pid: Pid) -> std::io::Result<Self> {
        Ok(Self(sys::process::Child::from_pid(pid)?))
    }

    /// Waits until the process was terminated.
    ///
    /// ## Cancel Safety
    /// This method is cancel safe.
    ///
    /// ## Errors
    /// An `Err(_)` is returned if the underlying OS function failed.
    pub async fn wait(&self) -> Result<Wait, WaitError> {
        self.0.wait().await.map_err(Into::into)
    }

    /// Sends the specified signal to the child process.
    ///
    /// ## Errors
    /// An `Err(_)` is returned if the underlying OS function failed.
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

/// Spawns a process associated with given [`std::process::Command`], returning a [`Child`] object.
///
/// ## Errors
/// An `Err(_)` is returned if the underlying OS function failed.
pub async fn spawn(cmd: &mut std::process::Command) -> std::io::Result<Child> {
    Ok(Child::from_std(cmd.spawn()?))
}

#[derive(Debug, Clone, Default)]
pub enum Stdio {
    /// The child inherits from the corresponding parent descriptor.
    #[default]
    Inherit,

    /// A new pipe should be arranged to connect the parent and child processes.
    Piped,

    File(PathBuf),
}
impl Stdio {
    pub async fn to_std(&self) -> std::io::Result<std::process::Stdio> {
        Ok(match self {
            Self::Inherit => std::process::Stdio::inherit(),
            Self::Piped => std::process::Stdio::piped(),
            Self::File(path) => tokio::fs::File::options()
                .append(true)
                .create(true)
                .open(path)
                .await?
                .into_std()
                .await
                .into(),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommandEnv {
    pub(crate) uid: Option<libc::uid_t>,
    pub(crate) gid: Option<libc::gid_t>,
    pub(crate) groups: Option<Vec<libc::gid_t>>,
    pub(crate) clear_vars: bool,
    pub(crate) vars: Vec<(OsString, Option<OsString>)>,
    pub(crate) stdout: Stdio,
    pub(crate) stderr: Stdio,
    pub(crate) working_dir: Option<PathBuf>,
    pub(crate) setsid: bool,
}
impl CommandEnv {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn uid<T: Into<Option<libc::uid_t>>>(&mut self, uid: T) -> &mut Self {
        if let Some(x) = uid.into() {
            self.uid = Some(x);
        }
        self
    }

    #[inline]
    pub fn gid<T: Into<Option<libc::gid_t>>>(&mut self, gid: T) -> &mut Self {
        if let Some(x) = gid.into() {
            self.gid = Some(x);
        }
        self
    }

    #[inline]
    pub fn groups<T: Into<Option<Vec<libc::gid_t>>>>(&mut self, groups: T) -> &mut Self {
        if let Some(x) = groups.into() {
            self.groups = Some(x);
        }
        self
    }

    #[inline]
    pub fn clear_vars(&mut self, val: bool) -> &mut Self {
        self.clear_vars = val;
        self
    }

    #[inline]
    pub fn var<K: Into<OsString>, V: Into<Option<C>>, C: Into<OsString>>(
        &mut self,
        k: K,
        v: V,
    ) -> &mut Self {
        self.vars.push((k.into(), v.into().map(Into::into)));
        self
    }

    #[inline]
    pub fn vars<
        I: Iterator<Item = (K, V)>,
        K: Into<OsString>,
        V: Into<Option<T>>,
        T: Into<OsString>,
    >(
        &mut self,
        iter: I,
    ) -> &mut Self {
        iter.for_each(|(k, v)| {
            self.var(k, v);
        });
        self
    }

    #[inline]
    pub fn working_dir<P: Into<PathBuf>, T: Into<Option<P>>>(&mut self, value: T) -> &mut Self {
        self.working_dir = value.into().map(Into::into);
        self
    }

    pub fn login<'a, U: Into<Option<&'a str>>>(&mut self, name: U) -> anyhow::Result<&mut Self> {
        if let Some(x) = name.into() {
            crate::sys::process::command_login(self, x)?;
        }
        Ok(self)
    }

    #[inline]
    pub fn stdout(&mut self, new: Stdio) -> &mut Self {
        self.stdout = new;
        self
    }

    #[inline]
    pub fn stderr(&mut self, new: Stdio) -> &mut Self {
        self.stderr = new;
        self
    }

    #[inline]
    pub fn setsid(&mut self, val: bool) -> &mut Self {
        self.setsid = val;
        self
    }
}

#[derive(Debug)]
pub struct Command {
    pub(crate) env: CommandEnv,
    pub(crate) program: OsString,
    pub(crate) arg0: Option<OsString>,
    pub(crate) args: Vec<OsString>,
}
impl Command {
    #[inline]
    pub fn new<S: Into<OsString>>(program: S) -> Self {
        Self {
            env: CommandEnv::default(),
            program: program.into(),
            arg0: None,
            args: vec![],
        }
    }

    #[inline]
    pub fn arg0<S: Into<OsString>>(&mut self, arg0: S) -> &mut Self {
        self.arg0 = Some(arg0.into());
        self
    }

    #[inline]
    pub fn arg<S: Into<OsString>>(&mut self, arg: S) -> &mut Self {
        self.args.push(arg.into());
        self
    }

    #[inline]
    pub async fn to_std(&self) -> anyhow::Result<std::process::Command> {
        crate::sys::process::command_to_std(self).await
    }

    #[inline]
    pub async fn spawn(&self) -> anyhow::Result<Child> {
        Ok(spawn(&mut self.to_std().await?).await?)
    }
}
impl Deref for Command {
    type Target = CommandEnv;

    fn deref(&self) -> &Self::Target {
        &self.env
    }
}
impl DerefMut for Command {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.env
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct WaitError(String);
impl From<sys::process::WaitError> for WaitError {
    fn from(value: sys::process::WaitError) -> Self {
        Self(value.to_string())
    }
}
