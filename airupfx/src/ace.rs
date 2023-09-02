//! # Airup Command Engine

use libc::SIGKILL;

use crate::{
    process::{ExitStatus, Pid, Wait, WaitError, SIGTERM},
    users::{find_user_by_name, Gid, Uid},
    util::BoxFuture,
};
use std::{
    borrow::Cow,
    collections::BTreeMap,
    ffi::OsString,
    os::unix::process::CommandExt,
    sync::{Arc, Mutex, MutexGuard},
    time::Duration,
};

/// The Airup Command Engine.
#[derive(Debug, Default)]
pub struct Ace {
    env: Mutex<Env>,
}
impl Ace {
    /// Creates a new [Ace] instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run(&self, cmd: &str) -> Result<Child, Error> {
        let tokens = cmd.split(' '); // placeholder impl, to be replaced
        self.run_tokenized(tokens.map(Cow::Borrowed)).await
    }

    pub async fn run_timeout(
        &self,
        cmd: &str,
        timeout: Option<Duration>,
    ) -> Result<Result<(), CommandExitError>, Error> {
        let mut child = self.run(cmd).await?;
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

    /// Returns a reference to the [Env] instance of this Ace engine.
    pub fn env(&self) -> MutexGuard<Env> {
        self.env.lock().unwrap()
    }

    /// Sets [Env] instance of this engine.
    pub fn set_env(&self, env: Env) {
        *self.env.lock().unwrap() = env;
    }

    fn run_tokenized<'a, I: Iterator<Item = Cow<'a, str>> + Send + Sync + 'a>(
        &'a self,
        mut tokens: I,
    ) -> BoxFuture<'_, Result<Child, Error>> {
        Box::pin(async {
            let cmd = tokens.next().ok_or(Error::CommandNotFound)?;
            if cmd == "always-success" {
                Ok(Child::AlwaysSuccess(Box::new(
                    self.run_tokenized(tokens)
                        .await
                        .unwrap_or_else(|_| Child::AlwaysSuccess(Box::new(Child::Nop))),
                )))
            } else if cmd == "async" {
                Ok(Child::Async(Box::new(self.run_tokenized(tokens).await?)))
            } else {
                self.run_bin_command(&cmd, tokens).await
            }
        })
    }

    async fn run_bin_command<'a, I: Iterator<Item = Cow<'a, str>> + Send + Sync>(
        &self,
        arg0: &str,
        args: I,
    ) -> Result<Child, Error> {
        let mut command = self.env.lock().unwrap().as_command(arg0);
        command.args(args.map(|x| OsString::from(&*x)));
        let _lock = crate::process::child_queue().lock_waiter().await;
        let child = command.spawn()?;
        Ok(Child::Process(crate::process::Child::from_std(child)))
    }
}

/// Environment of an ACE engine.
#[derive(Debug, Clone, Default)]
pub struct Env {
    uid: Option<Uid>,
    gid: Option<Gid>,
    // groups: Option<Vec<Gid>>, not implemented yet.
    clear_vars: bool,
    vars: BTreeMap<OsString, Option<OsString>>,
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
    pub fn user<T: Into<Option<String>>>(&mut self, name: T) -> Result<&mut Self, Error> {
        let name = name.into();
        let user = match &name {
            Some(x) => find_user_by_name(x).ok_or(Error::UserNotFound)?,
            None => return Ok(self),
        };
        Ok(self.uid(user.uid).gid(user.gid))
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

    #[cfg(feature = "files")]
    pub fn from_service_env(env: &crate::files::service::Env) -> Result<Self, Error> {
        let mut result = Self::new();

        result.user(env.user.clone())?.uid(env.uid).gid(env.gid);

        Ok(result)
    }

    fn as_command(&self, arg0: &str) -> std::process::Command {
        let mut command = std::process::Command::new(arg0);
        if let Some(x) = self.uid {
            command.uid(x as _);
        }
        if let Some(x) = self.gid {
            command.gid(x as _);
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
    }
}

/// Representation of a running or exited ACE child.
#[derive(Debug)]
pub enum Child {
    Async(Box<Self>),
    AlwaysSuccess(Box<Self>),
    Process(crate::process::Child),
    Nop,
}
impl Child {
    /// Returns process ID of the child.
    #[inline]
    pub fn id(&self) -> Pid {
        match self {
            Self::Async(child) => child.id(),
            Self::AlwaysSuccess(child) => child.id(),
            Self::Process(proc) => proc.id(),
            Self::Nop => 0,
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
                Self::Nop => Wait::new(0, ExitStatus::SUCCESS),
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
                Self::Nop => (),
            };

            Ok(())
        })
    }

    /// Attempts to kill the process with given signal number. If the process did not terminate in specified time, it will be
    /// forcefully killed using [SIGKILL].
    ///
    /// Note that this may take too long since that `kill()` may be blocking and it is uninterruptable.
    pub async fn kill_timeout(&mut self, sig: i32, timeout: Option<Duration>) -> Result<(), Error> {
        self.kill(sig).await?;
        match self.wait_timeout(timeout).await {
            Ok(_) => Ok(()),
            Err(err) => match err {
                Error::TimedOut => self.kill(SIGKILL).await,
                other => Err(other),
            },
        }
    }
}
impl From<crate::process::Child> for Child {
    fn from(value: crate::process::Child) -> Self {
        Self::Process(value)
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
