//! A module for working with processes.

use crate::io::PiperHandle;
use crate::sys;
use once_cell::sync::Lazy;
use std::{
    convert::Infallible,
    ffi::OsString,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::Arc,
};

/// The OS-assigned process identifier associated with this process.
pub static ID: Lazy<u32> = Lazy::new(std::process::id);

/// Reloads the process image with the version on the filesystem.
///
/// # Errors
/// An `Err(_)` is returned if the underlying OS function failed.
pub fn reload_image() -> std::io::Result<Infallible> {
    sys::process::reload_image()
}

/// Called when a fatal error has occured.
///
/// If the process has `pid == 1`, this will start a shell and reloads the process image. Otherwise this will make current
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

/// Describes the result of calling `wait`-series methods.
#[derive(Debug, Clone)]
pub struct Wait {
    pid: i64,
    pub exit_status: ExitStatus,
}
impl Wait {
    /// Creates a new [`Wait`] object.
    pub fn new(pid: i64, exit_status: ExitStatus) -> Self {
        Self { pid, exit_status }
    }

    /// Returns the OS-assigned process identifier associated with the wait result.
    pub fn pid(&self) -> i64 {
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
    pub const fn id(&self) -> i64 {
        self.0.id() as _
    }

    /// Creates a [`Child`] instance from PID.
    ///
    /// # Errors
    /// An `Err(_)` is returned if the process is not a valid child process of current process.
    pub fn from_pid(pid: i64) -> std::io::Result<Self> {
        Ok(Self(sys::process::Child::from_pid(pid as _)?))
    }

    /// Waits until the process was terminated.
    ///
    /// # Cancel Safety
    /// This method is cancel safe.
    ///
    /// # Errors
    /// An `Err(_)` is returned if the underlying OS function failed.
    pub async fn wait(&self) -> Result<Wait, WaitError> {
        self.0.wait().await.map_err(Into::into)
    }

    /// Sends the specified signal to the child process.
    ///
    /// # Errors
    /// An `Err(_)` is returned if the underlying OS function failed.
    pub fn send_signal(&self, sig: i32) -> std::io::Result<()> {
        self.0.send_signal(sig)
    }

    /// Kills the child process.
    ///
    /// # Errors
    /// An `Err(_)` is returned if the underlying OS function failed.
    pub fn kill(&self) -> std::io::Result<()> {
        self.0.kill()
    }

    /// Returns a reference to the `stdout` piper handle of the child process.
    pub fn stdout(&self) -> Option<Arc<PiperHandle>> {
        self.0.stdout()
    }

    /// Returns a reference to the `stderr` piper handle of the child process.
    pub fn stderr(&self) -> Option<Arc<PiperHandle>> {
        self.0.stderr()
    }
}
impl From<crate::sys::process::Child> for Child {
    fn from(inner: crate::sys::process::Child) -> Self {
        Self(inner)
    }
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
    pub(crate) uid: Option<u32>,
    pub(crate) gid: Option<u32>,
    pub(crate) groups: Option<Vec<u32>>,
    pub(crate) clear_vars: bool,
    pub(crate) vars: Vec<(OsString, Option<OsString>)>,
    pub(crate) stdout: Stdio,
    pub(crate) stderr: Stdio,
    pub(crate) working_dir: Option<PathBuf>,
    pub(crate) setsid: bool,
    pub(crate) cpu_limit: Option<u64>,
    pub(crate) mem_limit: Option<u64>,
}
impl CommandEnv {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn uid<T: Into<Option<u32>>>(&mut self, uid: T) -> &mut Self {
        if let Some(x) = uid.into() {
            self.uid = Some(x);
        }
        self
    }

    #[inline]
    pub fn gid<T: Into<Option<u32>>>(&mut self, gid: T) -> &mut Self {
        if let Some(x) = gid.into() {
            self.gid = Some(x);
        }
        self
    }

    #[inline]
    pub fn groups<T: Into<Option<Vec<u32>>>>(&mut self, groups: T) -> &mut Self {
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

    #[inline]
    pub fn cpu_limit<T: Into<Option<u64>>>(&mut self, val: T) -> &mut Self {
        self.cpu_limit = val.into();
        self
    }

    #[inline]
    pub fn mem_limit<T: Into<Option<u64>>>(&mut self, val: T) -> &mut Self {
        self.mem_limit = val.into();
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
    pub async fn spawn(&self) -> anyhow::Result<Child> {
        Ok(crate::sys::process::spawn(self).await?.into())
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
