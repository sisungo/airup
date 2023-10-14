//! # Airup Command Engine

pub mod builtins;
pub mod parser;

use crate::{
    env::with_user_by_name,
    process::{ExitStatus, Pid, Wait, WaitError},
    signal::{SIGKILL, SIGTERM},
    std_port::CommandExt as _,
    util::BoxFuture,
};
use ahash::AHashMap;
use std::{
    collections::BTreeMap, ffi::OsString, os::unix::process::CommandExt as _, path::PathBuf,
    sync::Arc, time::Duration,
};
use sysinfo::{Gid, Uid, UserExt};
use tokio::{io::AsyncRead, sync::mpsc};

/// The Airup Command Engine.
#[derive(Debug, Default)]
pub struct Ace {
    pub env: Env,
    pub modules: Modules,
}
impl Ace {
    /// Creates a new [Ace] instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Runs the given command, returning the child.
    pub async fn run(&self, cmd: &str) -> Result<Child, Error> {
        if let Some(x) = cmd.strip_prefix("sh.exec") {
            self.run_tokenized(["sh".into(), "-c".into(), x.into()].into_iter())
                .await
        } else {
            self.run_tokenized(parser::tokenize(cmd).into_iter()).await
        }
    }

    /// Runs the given command and waits until it completed.
    pub async fn run_wait(&self, cmd: &str) -> Result<Result<(), CommandExitError>, Error> {
        self.run_wait_timeout(cmd, None).await
    }

    /// Runs the given command and waits until it completed or a timeout expired.
    pub async fn run_wait_timeout(
        &self,
        cmd: &str,
        timeout: Option<Duration>,
    ) -> Result<Result<(), CommandExitError>, Error> {
        let child = self.run(cmd).await?;
        match child.wait_timeout(timeout).await {
            Ok(wait) => Ok(CommandExitError::from_wait(&wait)),
            Err(err) => match err {
                Error::TimedOut => {
                    child.kill_timeout(SIGTERM, timeout).await?;
                    Err(Error::TimedOut)
                }
                other => Err(other),
            },
        }
    }

    fn run_tokenized<'a, I: Iterator<Item = String> + Send + Sync + 'a>(
        &'a self,
        tokens: I,
    ) -> BoxFuture<'_, Result<Child, Error>> {
        Box::pin(async {
            let cmd: parser::Command = tokens.into();
            if cmd.module == "-" {
                Ok(Child::AlwaysSuccess(Box::new(
                    self.run_tokenized(cmd.args.into_iter())
                        .await
                        .unwrap_or_else(|_| {
                            Child::AlwaysSuccess(Box::new(Child::Builtin(
                                builtins::noop(vec![]).into(),
                            )))
                        }),
                )))
            } else if cmd.module == "&" {
                Ok(Child::Async(Box::new(
                    self.run_tokenized(cmd.args.into_iter()).await?,
                )))
            } else if let Some(x) = self.modules.builtins.get(&cmd.module[..]) {
                Ok(Child::Builtin(tokio::sync::Mutex::new(x(cmd.args))))
            } else {
                self.run_bin_command(&cmd).await
            }
        })
    }

    async fn run_bin_command(&self, cmd: &parser::Command) -> Result<Child, Error> {
        let mut command = self.env.as_command(&cmd.module).await?;
        command.args(cmd.args.iter().map(OsString::from));
        Ok(Child::Process(crate::process::spawn(&mut command).await?))
    }
}

#[derive(Debug, Clone)]
pub struct Modules {
    builtins: AHashMap<&'static str, builtins::BuiltinModule>,
}
impl Modules {
    pub fn new() -> Self {
        let mut builtins = AHashMap::with_capacity(32);
        builtins::init(&mut builtins);
        Self { builtins }
    }
}
impl Default for Modules {
    fn default() -> Self {
        Self::new()
    }
}

/// Environment of an ACE engine.
#[derive(Debug, Clone, Default)]
pub struct Env {
    uid: Option<Uid>,
    gid: Option<Gid>,
    groups: Option<Vec<Gid>>,
    clear_vars: bool,
    vars: BTreeMap<OsString, Option<OsString>>,
    stdout: Stdio,
    stderr: Stdio,
    working_dir: Option<PathBuf>,
    setsid: bool,
}
impl Env {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn uid<T: Into<Option<Uid>>>(&mut self, uid: T) -> &mut Self {
        if let Some(x) = uid.into() {
            self.uid = Some(x);
        }
        self
    }

    #[inline]
    pub fn gid<T: Into<Option<Gid>>>(&mut self, gid: T) -> &mut Self {
        if let Some(x) = gid.into() {
            self.gid = Some(x);
        }
        self
    }

    #[inline]
    pub fn groups<T: Into<Option<Vec<Gid>>>>(&mut self, groups: T) -> &mut Self {
        if let Some(x) = groups.into() {
            self.groups = Some(x);
        }
        self
    }

    #[inline]
    pub async fn user<T: Into<Option<String>>>(&mut self, name: T) -> Result<&mut Self, Error> {
        let name = match name.into() {
            Some(x) => x,
            None => return Ok(self),
        };
        let (uid, gid) =
            with_user_by_name(&name, |user| (user.id().clone(), user.group_id().clone()))
                .await
                .ok_or(Error::UserNotFound)?;
        Ok(self.uid(uid).gid(gid))
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
        self.vars.insert(k.into(), v.into().map(Into::into));
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

    async fn as_command(&self, arg0: &str) -> Result<std::process::Command, Error> {
        let mut command = std::process::Command::new(arg0);
        if let Some(x) = &self.uid {
            command.uid(**x);
        }
        if let Some(x) = &self.gid {
            command.gid(**x);
        }
        if let Some(x) = &self.working_dir {
            command.current_dir(x);
        }
        if self.clear_vars {
            command.env_clear();
        }
        self.vars.iter().for_each(|(k, v)| match v {
            Some(v) => {
                command.env(k, v);
            }
            None => {
                command.env_remove(k);
            }
        });
        command
            .stdout(self.stdout.to_std().await?)
            .stderr(self.stderr.to_std().await?);
        if self.setsid {
            command.setsid();
        }

        Ok(command)
    }
}

/// Representation of a running or exited ACE child.
#[derive(Debug)]
pub enum Child {
    Async(Box<Self>),
    AlwaysSuccess(Box<Self>),
    Process(crate::process::Child),
    Builtin(tokio::sync::Mutex<mpsc::Receiver<i32>>),
}
impl Child {
    /// Returns process ID of the child.
    #[inline]
    pub fn id(&self) -> Pid {
        match self {
            Self::Async(child) => child.id(),
            Self::AlwaysSuccess(child) => child.id(),
            Self::Process(proc) => proc.id(),
            Self::Builtin(_) => 0,
        }
    }

    /// Waits until the task completed.
    #[inline]
    pub fn wait(&self) -> BoxFuture<Result<Wait, Error>> {
        Box::pin(async move {
            Ok(match self {
                Self::Async(child) => Wait::new(child.id(), ExitStatus::SUCCESS),
                Self::AlwaysSuccess(child) => {
                    let mut wait = child.wait().await?;
                    wait.exit_status = ExitStatus::SUCCESS;
                    wait
                }
                Self::Process(proc) => proc.wait().await?,
                Self::Builtin(rx) => Wait::new(0, builtins::wait(&mut *rx.lock().await).await),
            })
        })
    }

    /// Waits until the task completed. Returns [`Error::TimedOut`] if the specified timeout expired.
    pub async fn wait_timeout(&self, timeout: Option<Duration>) -> Result<Wait, Error> {
        let timeout = match timeout {
            Some(x) => x,
            None => return self.wait().await,
        };
        if timeout.is_zero() {
            return self.wait().await;
        }

        match tokio::time::timeout(timeout, self.wait()).await {
            Ok(x) => Ok(x?),
            Err(_) => {
                if matches!(self, Self::AlwaysSuccess(_)) {
                    Ok(Wait::new(self.id(), ExitStatus::SUCCESS))
                } else {
                    Err(Error::TimedOut)
                }
            }
        }
    }

    /// Kills the task.
    #[inline]
    pub fn kill(&self, sig: i32) -> BoxFuture<Result<(), Error>> {
        Box::pin(async move {
            match self {
                Self::Async(child) => child.kill(sig).await?,
                Self::AlwaysSuccess(child) => child.kill(sig).await?,
                Self::Process(proc) => proc.kill(sig).await?,
                Self::Builtin(_) => (),
            };

            Ok(())
        })
    }

    /// Attempts to kill the process with given signal number. If the process did not terminate in specified time, it will be
    /// forcefully killed using [SIGKILL].
    ///
    /// Note that this may take too long since that `kill()` may be blocking and it is uninterruptable.
    pub async fn kill_timeout(&self, sig: i32, timeout: Option<Duration>) -> Result<(), Error> {
        self.kill(sig).await?;
        match self.wait_timeout(timeout).await {
            Ok(_) => Ok(()),
            Err(err) => match err {
                Error::TimedOut => self.kill(SIGKILL).await,
                other => Err(other),
            },
        }
    }

    pub fn take_stdout(&mut self) -> Option<Box<dyn AsyncRead + Send + Sync>> {
        match self {
            Self::AlwaysSuccess(child) => child.take_stdout(),
            Self::Async(child) => child.take_stdout(),
            Self::Builtin(_) => None,
            Self::Process(child) => match child.take_stdout() {
                Some(x) => Some(Box::new(x)),
                None => None,
            },
        }
    }

    pub fn take_stderr(&mut self) -> Option<Box<dyn AsyncRead + Send + Sync>> {
        match self {
            Self::AlwaysSuccess(child) => child.take_stderr(),
            Self::Async(child) => child.take_stderr(),
            Self::Builtin(_) => None,
            Self::Process(child) => match child.take_stderr() {
                Some(x) => Some(Box::new(x)),
                None => None,
            },
        }
    }
}
impl From<crate::process::Child> for Child {
    fn from(value: crate::process::Child) -> Self {
        Self::Process(value)
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
                .open(path)
                .await?
                .into_std()
                .await
                .into(),
        })
    }
}

/// An error occured by ACE operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("user not found")]
    UserNotFound,

    #[error("command not found")]
    CommandNotFound,

    #[error("wait() failed: {0}")]
    Wait(WaitError),

    #[error("{0}")]
    Io(Arc<std::io::Error>),

    #[error("timed out")]
    TimedOut,
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.into())
    }
}
impl From<WaitError> for Error {
    fn from(value: WaitError) -> Self {
        Self::Wait(value)
    }
}

/// An error that the command failed.
#[derive(Debug, Clone, thiserror::Error)]
pub enum CommandExitError {
    #[error("command exited with code {0}")]
    Exited(i32),

    #[error("command was terminated by signal {0}")]
    Signaled(i32),
}
impl CommandExitError {
    pub fn from_wait(wait: &Wait) -> Result<(), Self> {
        match wait.code() {
            Some(code) => match code {
                0 => Ok(()),
                x => Err(Self::Exited(x)),
            },
            None => Err(Self::Signaled(wait.signal().unwrap())),
        }
    }

    pub fn from_wait_force(wait: &Wait) -> Self {
        match wait.code() {
            Some(code) => Self::Exited(code),
            None => Self::Signaled(wait.signal().unwrap()),
        }
    }
}