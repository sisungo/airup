//! A module for working with processes.

cfg_if::cfg_if! {
    if #[cfg(target_family = "unix")] {
        #[path = "unix.rs"]
        mod sys;
    } else {
        std::compile_error!("This target is not supported by `Airup` yet. Consider opening an issue at https://github.com/sisungo/airup/issues?");
    }
}

use airupfx_io::line_piper::Callback as LinePiperCallback;
use std::{convert::Infallible, ffi::OsString, path::PathBuf};

/// Returns `true` if supervising `forking` services are supported on the system.
pub fn is_forking_supervisable() -> bool {
    sys::is_forking_supervisable()
}

/// Returns `true` if the supervisor should run as it was running as `pid = 1` on Unix.
pub fn as_pid1() -> bool {
    std::process::id() == 1
}

/// Temporarily disables `airupfx`'s process manager managing child processes.
///
/// This is useful avoiding TOCTOU-like attacks, or manually managing child processes.
pub async fn lock() -> impl Drop {
    sys::lock().await
}

/// Reloads current process' executable image.
pub fn reload_image() -> std::io::Result<Infallible> {
    sys::reload_image()
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
pub struct Child(sys::Child);
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
        Ok(Self(sys::Child::from_pid(pid as _)?))
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
    pub async fn send_signal(&self, sig: i32) -> std::io::Result<()> {
        self.0.send_signal(sig).await
    }

    /// Kills the child process.
    ///
    /// # Errors
    /// An `Err(_)` is returned if the underlying OS function failed.
    pub async fn kill(&self) -> std::io::Result<()> {
        self.0.kill().await
    }
}
impl From<sys::Child> for Child {
    fn from(inner: sys::Child) -> Self {
        Self(inner)
    }
}

#[derive(Default)]
pub enum Stdio {
    /// Redirects `stdio` to the null device.
    Nulldev,

    /// The child inherits from the corresponding parent descriptor.
    #[default]
    Inherit,

    /// Similar to [`Stdio::Piped`], but a callback is called on each line.
    Callback(Box<dyn LinePiperCallback>),

    /// The child's stdio is redirected to the file.
    File(PathBuf),
}
impl Clone for Stdio {
    fn clone(&self) -> Self {
        match self {
            Self::Nulldev => Self::Nulldev,
            Self::Inherit => Self::Inherit,
            Self::Callback(c) => Self::Callback(c.clone_boxed()),
            Self::File(f) => Self::File(f.clone()),
        }
    }
}
impl Stdio {
    pub async fn to_std(&self) -> std::io::Result<std::process::Stdio> {
        Ok(match self {
            Self::Nulldev => std::process::Stdio::null(),
            Self::Inherit => std::process::Stdio::inherit(),
            Self::Callback(_) => std::process::Stdio::piped(),
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

/// Cross-platform representation of a child process' environment.
#[derive(Clone, Default)]
pub struct CommandEnv {
    pub uid: Option<u32>,
    pub gid: Option<u32>,
    pub groups: Option<Vec<u32>>,
    pub clear_vars: bool,
    pub vars: Vec<(OsString, Option<OsString>)>,
    pub stdin: Stdio,
    pub stdout: Stdio,
    pub stderr: Stdio,
    pub working_dir: Option<PathBuf>,
    pub setsid: bool,
    pub cpu_limit: Option<u64>,
    pub mem_limit: Option<u64>,
    pub root_dir: Option<PathBuf>,
}
impl CommandEnv {
    #[inline]
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
    pub fn var<C: Into<OsString>>(
        &mut self,
        k: impl Into<OsString>,
        v: impl Into<Option<C>>,
    ) -> &mut Self {
        self.vars.push((k.into(), v.into().map(Into::into)));
        self
    }

    #[inline]
    pub fn vars<K: Into<OsString>, V: Into<Option<T>>, T: Into<OsString>>(
        &mut self,
        iter: impl Iterator<Item = (K, V)>,
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

    #[inline]
    pub fn root_dir<P: Into<PathBuf>, T: Into<Option<P>>>(&mut self, value: T) -> &mut Self {
        self.root_dir = value.into().map(Into::into);
        self
    }

    #[inline]
    pub fn login<'a, U: Into<Option<&'a str>>>(&mut self, name: U) -> std::io::Result<&mut Self> {
        if let Some(x) = name.into() {
            sys::command_login(self, x)?;
        }
        Ok(self)
    }

    #[inline]
    pub fn stdin(&mut self, new: Stdio) -> &mut Self {
        self.stdin = new;
        self
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

/// Cross-platform representation of creation of a child process.
pub struct Command {
    pub env: CommandEnv,
    pub program: OsString,
    pub arg0: Option<OsString>,
    pub args: Vec<OsString>,
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
    pub fn stdin(&mut self, new: Stdio) -> &mut Self {
        self.env.stdin(new);
        self
    }

    #[inline]
    pub fn stdout(&mut self, new: Stdio) -> &mut Self {
        self.env.stdout(new);
        self
    }

    #[inline]
    pub fn stderr(&mut self, new: Stdio) -> &mut Self {
        self.env.stderr(new);
        self
    }

    #[inline]
    pub async fn spawn(&self) -> std::io::Result<Child> {
        Ok(sys::spawn(self).await?.into())
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{0}")]
pub struct WaitError(String);
impl From<sys::WaitError> for WaitError {
    fn from(value: sys::WaitError) -> Self {
        Self(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn spawn_and_wait() {
        let spawn_wait = move |x| async move {
            crate::Command::new(x)
                .spawn()
                .await
                .unwrap()
                .wait()
                .await
                .unwrap()
        };

        assert!(spawn_wait("true").await.is_success());
        assert!(!spawn_wait("false").await.is_success());
    }

    #[tokio::test]
    async fn stdout_capture() {
        let (tx, mut rx) = tokio::sync::mpsc::channel(2);
        let callback = airupfx_io::line_piper::ChannelCallback::new(tx);
        crate::Command::new("echo")
            .arg("Hello, world!")
            .stdout(crate::Stdio::Callback(Box::new(callback)))
            .spawn()
            .await
            .unwrap()
            .wait()
            .await
            .unwrap();
        assert_eq!(rx.recv().await.as_deref(), Some(&b"Hello, world!\n"[..]));
    }
}
