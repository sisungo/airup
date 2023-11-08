//! Process management on Microsoft Windows.

use std::convert::Infallible;
use crate::process::Wait;

pub type Pid = libc::c_int;

/// Reloads the process image with the version on the filesystem.
pub fn reload_image() -> std::io::Result<Infallible> {
    std::process::Command::new(std::env::current_exe()?)
        .args(std::env::args_os().skip(1))
        .spawn()?
        .wait()?;
    Ok(std::process::exit(0))
}

/// Sends the given signal to the specified process.
///
/// # Errors
/// An `Err(_)` is returned if the underlying OS function failed.
pub async fn kill(pid: Pid, signum: i32) -> std::io::Result<()> {
    todo!()
}

pub(crate) fn command_login(
    env: &mut crate::process::CommandEnv,
    name: &str,
) -> anyhow::Result<()> {
    todo!()
}

/// Converts from [`crate::process::Command`] to [`std::process::Command`].
pub(crate) async fn command_to_std(
    command: &crate::process::Command,
) -> anyhow::Result<std::process::Command> {
    let mut result = std::process::Command::new(&command.program);
    command.args.iter().for_each(|x| {
        result.arg(x);
    });
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

    Ok(result)
}

/// Representation of a running or exited child process.
#[derive(Debug)]
pub struct Child;
impl Child {
    /// Returns OS-assign process ID of the child process.
    pub const fn id(&self) -> Pid {
        todo!()
    }

    /// Converts from [std::process::Child] to [Child].
    pub fn from_std(c: std::process::Child) -> Self {
        todo!()
    }

    /// Waits until the process was terminated.
    ///
    /// # Cancel Safety
    /// This method is cancel safe.
    pub async fn wait(&self) -> Result<Wait, WaitError> {
        todo!()
    }

    /// Sends the specified signal to the child process.
    ///
    /// # Errors
    /// An `Err(_)` is returned if the underlying OS function failed.
    pub async fn kill(&self, _: i32) -> std::io::Result<()> {
        todo!()
    }

    pub unsafe fn from_pid_unchecked(
        pid: Pid,
    ) -> Self {
        todo!()
    }

    /// Creates a [`Child`] instance from PID.
    ///
    /// # Errors
    /// An `Err(_)` is returned if the process is not a valid child process of current process.
    pub fn from_pid(pid: Pid) -> std::io::Result<Self> {
        todo!()
    }
}

/// An error occured by calling `wait` on a [`Child`].
#[derive(Debug, Clone, thiserror::Error)]
#[error("todo")]
pub struct WaitError;