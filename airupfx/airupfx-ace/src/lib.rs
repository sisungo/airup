//! ACE, short of Airup Command Engine, is the builtin shell-like command language of Airup.

pub mod builtins;
pub mod parser;

use ahash::AHashMap;
use airup_sdk::error::IntoApiError;
use airupfx_isolator::Realm;
use airupfx_process::{CommandEnv, ExitStatus, Wait, WaitError};
use libc::SIGTERM;
use std::{future::Future, pin::Pin, sync::Arc, time::Duration};
use tokio::task::JoinHandle;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The Airup Command Engine.
#[derive(Default)]
pub struct Ace {
    pub env: CommandEnv,
    pub realm: Option<Arc<Realm>>,
    modules: Modules,
}
impl Ace {
    /// Creates a new [`Ace`] instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Runs the given command, returning the child.
    pub async fn run(&self, cmd: &str) -> Result<Child, Error> {
        if let Some(x) = cmd.strip_prefix("sh.run") {
            self.run_tokenized(["sh".into(), "-c".into(), x.into()].into_iter())
                .await
        } else {
            self.run_tokenized(shlex::split(cmd).ok_or(Error::ParseError)?.into_iter())
                .await
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

    fn run_tokenized<'a>(
        &'a self,
        tokens: impl Iterator<Item = String> + Send + 'a,
    ) -> BoxFuture<'_, Result<Child, Error>> {
        Box::pin(async {
            let cmd: parser::Command = tokens.into();
            if cmd.module == "-" {
                let otherwise = |_| {
                    Child::AlwaysSuccess(Box::new(Child::Builtin(builtins::noop(vec![]).into())))
                };
                Ok(Child::AlwaysSuccess(Box::new(
                    self.run_tokenized(cmd.args.into_iter())
                        .await
                        .unwrap_or_else(otherwise),
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
        let mut command = airupfx_process::Command::new(&cmd.module);
        cmd.args.iter().for_each(|x| {
            command.arg(x);
        });
        command.env = self.env.clone();
        let child = command.spawn().await?;
        if let Some(realm) = &self.realm {
            realm.add(child.id())?;
        }
        Ok(Child::Process(child))
    }
}

#[derive(Debug, Clone)]
struct Modules {
    builtins: AHashMap<&'static str, builtins::BuiltinModule>,
}
impl Modules {
    fn new() -> Self {
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

/// Representation of a running or exited ACE child.
#[derive(Debug)]
pub enum Child {
    Async(Box<Self>),
    AlwaysSuccess(Box<Self>),
    Process(airupfx_process::Child),
    Builtin(tokio::sync::Mutex<JoinHandle<i32>>),
}
impl Child {
    /// Returns process ID of the child.
    pub const fn id(&self) -> i64 {
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
                Self::Builtin(builtin) => {
                    Wait::new(0, builtins::wait(&mut *builtin.lock().await).await)
                }
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

    /// Sends a signal to the task.
    #[inline]
    pub fn send_signal(&self, sig: i32) -> BoxFuture<Result<(), Error>> {
        Box::pin(async move {
            match self {
                Self::Async(child) => child.send_signal(sig).await?,
                Self::AlwaysSuccess(child) => child.send_signal(sig).await?,
                Self::Process(proc) => proc.send_signal(sig)?,
                Self::Builtin(builtin) => builtin.lock().await.abort(),
            };

            Ok(())
        })
    }

    /// Kills the task.
    #[inline]
    pub fn kill(&self) -> BoxFuture<Result<(), Error>> {
        Box::pin(async move {
            match self {
                Self::Async(child) => child.kill().await?,
                Self::AlwaysSuccess(child) => child.kill().await?,
                Self::Process(proc) => proc.kill()?,
                Self::Builtin(builtin) => builtin.lock().await.abort(),
            };

            Ok(())
        })
    }

    /// Attempts to kill the process with given signal number. If the process did not terminate in specified time, it will be
    /// forcefully killed.
    pub async fn kill_timeout(&self, sig: i32, timeout: Option<Duration>) -> Result<(), Error> {
        self.send_signal(sig).await?;
        match self.wait_timeout(timeout).await {
            Ok(_) => Ok(()),
            Err(err) => match err {
                Error::TimedOut => self.kill().await,
                other => Err(other),
            },
        }
    }
}
impl From<airupfx_process::Child> for Child {
    fn from(value: airupfx_process::Child) -> Self {
        Self::Process(value)
    }
}

/// An error occured by ACE operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("parse error")]
    ParseError,

    #[error("wait() failed: {0}")]
    Wait(WaitError),

    #[error("{0}")]
    Io(String),

    #[error("operation timed out")]
    TimedOut,
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}
impl From<WaitError> for Error {
    fn from(value: WaitError) -> Self {
        Self::Wait(value)
    }
}
impl IntoApiError for Error {
    fn into_api_error(self) -> airup_sdk::Error {
        match self {
            Self::ParseError => airup_sdk::Error::AceParseError,
            Self::Wait(err) => airup_sdk::Error::internal(err.to_string()),
            Self::Io(message) => airup_sdk::Error::Io { message },
            Self::TimedOut => airup_sdk::Error::TimedOut,
        }
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
impl IntoApiError for CommandExitError {
    fn into_api_error(self) -> airup_sdk::Error {
        match self {
            Self::Exited(exit_code) => airup_sdk::Error::Exited { exit_code },
            Self::Signaled(signum) => airup_sdk::Error::Signaled { signum },
        }
    }
}
