//! Process management on Microsoft Windows.

use crate::process::{ExitStatus, Wait};
use std::convert::Infallible;

pub type Pid = libc::c_int;

pub fn reload_image() -> std::io::Result<Infallible> {
    std::process::Command::new(std::env::current_exe()?)
        .args(std::env::args_os().skip(1))
        .spawn()?
        .wait()?;
    Ok(std::process::exit(0))
}

pub(crate) fn command_login(
    env: &mut crate::process::CommandEnv,
    name: &str,
) -> anyhow::Result<()> {
    todo!()
}

pub(crate) async fn command_to_tokio(
    command: &crate::process::Command,
) -> anyhow::Result<tokio::process::Command> {
    let mut result = tokio::process::Command::new(&command.program);
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

pub async fn spawn(cmd: &crate::process::Command) -> anyhow::Result<Child> {
    Ok(Child(command_to_tokio(cmd).await?.spawn()?.into()))
}

#[derive(Debug)]
pub struct Child(tokio::sync::RwLock<tokio::process::Child>);
impl Child {
    pub const fn id(&self) -> Pid {
        todo!()
    }

    pub async fn wait(&self) -> Result<Wait, WaitError> {
        self.0
            .write()
            .await
            .wait()
            .await
            .map(|x| Wait::new(0, ExitStatus::Exited(x.code().unwrap())))
    }

    pub async fn send_signal(&self, _: i32) -> std::io::Result<()> {
        Err(std::io::ErrorKind::Unsupported.into())
    }

    pub async fn kill(&self) -> std::io::Result<()> {
        self.0.write().await.kill().await
    }

    pub unsafe fn from_pid_unchecked(pid: Pid) -> Self {
        unimplemented!()
    }

    pub fn from_pid(pid: Pid) -> std::io::Result<Self> {
        Err(std::io::ErrorKind::Unsupported.into())
    }
}

pub type WaitError = std::io::Error;
