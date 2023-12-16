//! # Airup Service File Format
//! This module contains [`Service`], the main file format of an Airup service and its combinations.

use super::ReadError;
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

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<PathBuf>,

    #[serde(default)]
    pub service: Metadata,

    pub exec: Exec,

    #[serde(default)]
    pub env: Env,

    #[serde(default)]
    pub retry: Retry,

    #[serde(default)]
    pub watchdog: Watchdog,
}
impl Service {
    /// Reads multiple [`Service`]'s from given paths, then merge them into a single [`Service`] instance. The first element in
    /// parameter `paths` is seen as the "main".
    ///
    /// # Panics
    /// Panics if parameter `paths` is empty.
    pub async fn read_merge(paths: &[impl AsRef<Path>]) -> Result<Self, ReadError> {
        if paths.is_empty() {
            panic!("parameter `paths` must not be empty");
        }

        let main_path = paths.first().unwrap().as_ref();
        let main = tokio::fs::read_to_string(main_path).await?;
        let mut main: serde_json::Value = toml::from_str(&main)?;

        for path in &paths[1..] {
            let path = path.as_ref();
            let content = tokio::fs::read_to_string(path).await?;
            let patch: serde_json::Value = toml::from_str(&content)?;
            json_patch::merge(&mut main, &patch);
        }

        let mut object: Self = serde_json::from_value(main)?;

        object.validate()?;
        object.name = main_path.file_stem().unwrap().to_string_lossy().into();
        object.paths = paths.iter().map(|x| x.as_ref().to_path_buf()).collect();

        Ok(object)
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

    /// Returns the name to display for this service.
    pub fn display_name(&self) -> &str {
        self.service.display_name.as_deref().unwrap_or(&self.name)
    }
}

/// Executation environment of a service.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Env {
    /// User to execute for the service.
    pub user: Option<String>,

    /// UID to execute for the service.
    pub uid: Option<u32>,

    /// GID to execute for the service.
    pub gid: Option<u32>,

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

/// Representation of Standard I/O redirection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stdio {
    /// Inherits `stdio` from the parent process.
    Inherit,

    /// Redirects `stdio` to the specified file.
    File(PathBuf),

    /// Use the Airup logger to record `stdio` outputs.
    #[default]
    Log,
}

/// Represents to `[service]` section in a service TOML file.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Metadata {
    /// Display name of the service.
    pub display_name: Option<String>,

    /// Description of the service.
    pub description: Option<String>,

    /// Homepage of the service.
    pub homepage: Option<String>,

    /// Documentation of the service.
    pub docs: Option<String>,

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

/// Executation of a service, like start, stop, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Exec {
    /// Command to be executed before starting the service
    pub pre_start: Option<String>,

    /// Command to start the service
    pub start: String,

    /// Command to be executed after starting the service
    pub post_start: Option<String>,

    /// Command to reload the service
    pub reload: Option<String>,

    /// Command to be executed before stopping the service
    pub pre_stop: Option<String>,

    /// Command to stop the service
    pub stop: Option<String>,

    /// Command to be executed after stopping the service
    pub post_stop: Option<String>,

    /// Command to check health of the service
    pub health_check: Option<String>,

    /// Timeout of executing commands, in milliseconds
    all_timeout: Option<u64>,

    /// Timeout of starting the service until it's active, in milliseconds
    start_timeout: Option<u64>,

    /// Timeout of stopping the service, in milliseconds
    stop_timeout: Option<u64>,

    /// Timeout of checking health of a service, in milliseconds
    health_check_timeout: Option<u64>,

    /// Timeout of reloading the service, in milliseconds
    reload_timeout: Option<u64>,
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

    #[inline]
    pub fn health_check_timeout(&self) -> Option<Duration> {
        self.health_check_timeout
            .or(self.all_timeout)
            .map(Duration::from_millis)
    }
}

/// Retry conditions of a service.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Retry {
    /// Maximum attempts to retry executing
    #[serde(default)]
    pub max_attempts: i32,

    /// Delay time of retrying the service, in milliseconds
    #[serde(default)]
    pub delay: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Watchdog {
    /// Also mark the service failed on successful exits (`$? == 0`)
    #[serde(default)]
    pub successful_exit: bool,
}
