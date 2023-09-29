//! # Airup Service File Format
//! This module contains [Service], the main file format of an Airup service and its combinations.

use super::ReadError;
use airupfx::users::{Gid, Uid};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

/// An Airup service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,

    #[serde(default)]
    pub service: Metadata,

    pub exec: Exec,

    #[serde(default)]
    pub env: Env,

    #[serde(default)]
    pub helper: Vec<Helper>,
}
impl Service {
    pub const EXTENSION: &'static str = "airs";
    pub const SUFFIX: &'static str = ".airs";

    /// Reads a [Service] from given path.
    pub async fn read_from<P: AsRef<Path>>(path: P) -> Result<Self, ReadError> {
        Self::_read_from(path.as_ref()).await
    }

    /// Returns the name to display for this service.
    pub fn display_name(&self) -> &str {
        self.service.display_name.as_deref().unwrap_or(&self.name)
    }

    /// Returns `Ok(())` if the service is correct, otherwise returns `Err(_)`.
    pub fn validate(&self) -> Result<(), ReadError> {
        if self.env.user.is_some() && self.env.uid.is_some() {
            return Err("field `user` must not be set while `uid` is set".into());
        }
        match &self.service.pid_file {
            Some(_) => match &self.service.kind {
                Kind::Oneshot => {
                    Err("field `pid_file` must not be set with `kind=\"oneshot\"`".into())
                }
                _ => Ok(()),
            },
            None => match &self.service.kind {
                Kind::Forking => Err("field `pid_file` must be set with `kind=\"forking\"`".into()),
                _ => Ok(()),
            },
        }
    }

    async fn _read_from(path: &Path) -> Result<Self, ReadError> {
        let s = tokio::fs::read_to_string(path).await?;
        let mut object: Self = toml::from_str(&s)?;

        object.validate()?;
        object.name = path.file_stem().unwrap().to_string_lossy().into();

        Ok(object)
    }
}

/// Representation of environment of a service.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Env {
    /// User to execute for the service.
    pub user: Option<String>,

    /// UID to execute for the service.
    pub uid: Option<Uid>,

    /// GID to execute for the service.
    pub gid: Option<Gid>,

    /// Determines if environment variables from `airupd` process should be removed or not.
    #[serde(default)]
    pub clear_vars: bool,

    /// This field redirects standard output stream.
    #[serde(default)]
    pub stdout: Stdio,

    /// This field redirects standard error stream.
    #[serde(default)]
    pub stderr: Stdio,

    /// Working directory to start the service.
    pub working_dir: Option<PathBuf>,

    /// Environment variables to execute for the service.
    ///
    /// If a value is set to `null`, the environment variable gets removed if it exists.
    ///
    /// By default, the service runs with the same environment variables as `airupd`.
    #[serde(default)]
    pub vars: BTreeMap<String, Option<String>>,
}
impl Env {
    pub fn into_ace(&self) -> Result<airupfx::ace::Env, airupfx::ace::Error> {
        let mut result = airupfx::ace::Env::new();

        result
            .user(self.user.clone())?
            .uid(self.uid)
            .gid(self.gid)
            .stdout(self.stdout.clone().into_ace())
            .stderr(self.stderr.clone().into_ace())
            .clear_vars(self.clear_vars)
            .vars::<_, String, _, String>(self.vars.clone().into_iter())
            .working_dir::<PathBuf, _>(self.working_dir.clone());

        Ok(result)
    }
}

/// Representation of Standard I/O redirection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stdio {
    /// Inherits `stdio` from the parent process.
    #[default]
    Inherit,

    /// Redirects `stdio` to the specified file.
    File(PathBuf),
}
impl Stdio {
    pub fn into_ace(self) -> airupfx::ace::Stdio {
        match self {
            Self::Inherit => airupfx::ace::Stdio::Inherit,
            Self::File(path) => airupfx::ace::Stdio::File(path),
        }
    }
}

/// Represents to `[service]` section in a service TOML file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Metadata {
    /// Display name of the service.
    pub display_name: Option<String>,

    /// Description of the service.
    pub description: Option<String>,

    /// List of what the service can provide.
    #[serde(default)]
    pub provides: Vec<String>,

    /// Kind of the service.
    #[serde(default)]
    pub kind: Kind,

    /// Path of PID file of the service.
    pub pid_file: Option<PathBuf>,

    /// List of dependencies of the service.
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// List of services that conflicts with this service.
    #[serde(default)]
    pub conflicts_with: Vec<String>,
}

/// Kind of a service.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// The service process is running as the service is active.
    ///
    /// This conflicts with PID files.
    #[default]
    Simple,

    /// The process spawned by Airup forks and exits when it started successfully.
    ///
    /// A PID file must be specified with this.
    Forking,

    /// The service process will exit when it started and will keep active.
    ///
    /// This conflicts with PID files. If a stop command is not specified, the service will never be stopped.
    Oneshot,

    /// The service uses the Airup API to report its status.
    Notify,
}

/// Represents to commands related to the service, like start, stop, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Exec {
    /// Command to be executed before starting the service.
    pub pre_start: Option<String>,

    /// Command to start the service.
    pub start: String,

    /// Command to be executed after starting the service.
    pub post_start: Option<String>,

    /// Command to reload the service.
    pub reload: Option<String>,

    /// Command to be executed before stopping the service.
    pub pre_stop: Option<String>,

    /// Command to stop the service.
    pub stop: Option<String>,

    /// Command to be executed after stopping the service.
    pub post_stop: Option<String>,

    /// Timeout of executing commands, in milliseconds.
    all_timeout: Option<u64>,

    /// Timeout of starting the service until it's active.
    start_timeout: Option<u64>,

    /// Timeout of stopping the service.
    stop_timeout: Option<u64>,

    /// Timeout of reloading the service.
    reload_timeout: Option<u64>,

    /// Maximum times to retry executing.
    #[serde(default)]
    pub retry: i32,
}
impl Exec {
    #[inline]
    pub fn start_timeout(&self) -> Option<Duration> {
        self.start_timeout
            .or(self.all_timeout)
            .map(Duration::from_millis)
    }

    #[inline]
    pub fn stop_timeout(&self) -> Option<Duration> {
        self.stop_timeout
            .or(self.all_timeout)
            .map(Duration::from_millis)
    }

    #[inline]
    pub fn reload_timeout(&self) -> Option<Duration> {
        self.reload_timeout
            .or(self.all_timeout)
            .map(Duration::from_millis)
    }
}

/// Represents to `[[helper]]` section in a service TOML file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Helper {}
