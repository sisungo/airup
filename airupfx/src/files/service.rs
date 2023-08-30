//! Services

use super::ReadError;
use crate::users::{Gid, Uid};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

/// Represents to an Airup service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    #[serde(skip)]
    pub name: String,

    #[serde(default)]
    pub service: Metadata,

    pub exec: Exec,

    #[serde(default)]
    pub env: Env,
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
                Kind::Simple => {
                    Err("field `pid_file` must not be set with `kind=\"simple\"`".into())
                }
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

/// Represents to environment of a service.
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
    pub stdout: Option<String>,

    /// This field redirects standard error stream.
    pub stderr: Option<String>,

    /// Working directory to start the service.
    pub pwd: Option<PathBuf>,

    /// Environment variables to execute for the service.
    ///
    /// If a value is set to `null`, the environment variable gets removed if it exists.
    ///
    /// By default, the service runs with the same environment variables as `airupd`.
    #[serde(default)]
    pub vars: BTreeMap<String, Option<String>>,
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
}

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
