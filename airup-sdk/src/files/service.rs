//! # Airup Service File Format
//! This module contains [`Service`], the main file format of an Airup service and its combinations.

use super::{Named, ReadError, Validate};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, time::Duration};

/// An Airup service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Service {
    /// Name of the service.
    ///
    /// **NOTE**: This is an internal implementation detail and may subject to change in the future. This
    /// should **never** appear in any `.airs` files.
    #[doc(hidden)]
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,

    #[serde(default)]
    pub service: Metadata,

    pub exec: Exec,

    #[serde(default)]
    pub env: Env,

    #[serde(default)]
    pub retry: Retry,

    #[serde(default)]
    pub watchdog: Watchdog,

    #[serde(default)]
    pub reslimit: Reslimit,

    #[serde(default)]
    pub event_handlers: HashMap<String, String>,
}
impl Service {
    /// Returns the name to display for this service.
    pub fn display_name(&self) -> &str {
        self.service.display_name.as_deref().unwrap_or(&self.name)
    }
}
impl Validate for Service {
    fn validate(&self) -> Result<(), ReadError> {
        let env_user_conflict =
            self.env.login.is_some() && (self.env.uid.is_some() || self.env.gid.is_some());
        let oneshot_pid_file =
            self.service.pid_file.is_some() && matches!(self.service.kind, Kind::Oneshot);
        let forking_no_pid_file =
            self.service.pid_file.is_none() && matches!(self.service.kind, Kind::Forking);
        let stdin_log = matches!(self.env.stdin, Stdio::Log);

        if env_user_conflict {
            return Err("fields `env.user` conflicts with either `env.uid` or `env.gid`".into());
        }
        if oneshot_pid_file {
            return Err("field `service.pid_file` must not be set with `kind=\"oneshot\"`".into());
        }
        if forking_no_pid_file {
            return Err("field `service.pid_file` must be set with `kind=\"forking\"`".into());
        }
        if stdin_log {
            return Err("value of field `env.stdin` cannot be \"log\"".into());
        }

        Ok(())
    }
}
impl Named for Service {
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

/// Executation environment of a service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Env {
    /// Login user to execute for the service.
    pub login: Option<String>,

    /// UID to execute for the service.
    pub uid: Option<u32>,

    /// GID to execute for the service.
    pub gid: Option<u32>,

    /// Determines if environment variables from `airupd` process should be removed or not.
    #[serde(default)]
    pub clear_vars: bool,

    /// This field redirects standard input stream.
    #[serde(default = "Stdio::default_nulldev")]
    pub stdin: Stdio,

    /// This field redirects standard output stream.
    #[serde(default = "Stdio::default_log")]
    pub stdout: Stdio,

    /// This field redirects standard error stream.
    #[serde(default = "Stdio::default_log")]
    pub stderr: Stdio,

    /// Working directory to start the service.
    pub working_dir: Option<PathBuf>,

    /// Root directory to start the service.
    pub root_dir: Option<PathBuf>,

    /// Environment variables to execute for the service.
    ///
    /// If a value is set to `null`, the environment variable gets removed if it exists.
    ///
    /// By default, the service runs with the same environment variables as `airupd`.
    #[serde(default)]
    pub vars: HashMap<String, toml::Value>,
}
impl Default for Env {
    fn default() -> Self {
        Self {
            login: None,
            uid: None,
            gid: None,
            clear_vars: false,
            stdin: Stdio::Nulldev,
            stdout: Stdio::Log,
            stderr: Stdio::Log,
            working_dir: None,
            root_dir: None,
            vars: HashMap::default(),
        }
    }
}

/// Representation of Standard I/O redirection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Stdio {
    /// Redirects `stdio` to null device.
    Nulldev,

    /// Inherits `stdio` from the parent process.
    Inherit,

    /// Redirects `stdio` to the specified file.
    File(PathBuf),

    /// Use the Airup logger to record `stdio` outputs.
    Log,
}
impl Stdio {
    fn default_log() -> Self {
        Self::Log
    }

    fn default_nulldev() -> Self {
        Self::Nulldev
    }
}

/// Represents to `[service]` section in a service TOML file.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    pub all_timeout: Option<u32>,

    /// Timeout of starting the service until it's active, in milliseconds
    pub start_timeout: Option<u32>,

    /// Timeout of stopping the service, in milliseconds
    pub stop_timeout: Option<u32>,

    /// Timeout of checking health of a service, in milliseconds
    pub health_check_timeout: Option<u32>,

    /// Timeout of reloading the service, in milliseconds
    pub reload_timeout: Option<u32>,
}
impl Exec {
    #[inline]
    pub fn start_timeout(&self) -> Option<Duration> {
        self.start_timeout
            .or(self.all_timeout)
            .map(|x| x as u64)
            .map(Duration::from_millis)
    }

    #[inline]
    pub fn stop_timeout(&self) -> Option<Duration> {
        self.stop_timeout
            .or(self.all_timeout)
            .map(|x| x as u64)
            .map(Duration::from_millis)
    }

    #[inline]
    pub fn reload_timeout(&self) -> Option<Duration> {
        self.reload_timeout
            .or(self.all_timeout)
            .map(|x| x as u64)
            .map(Duration::from_millis)
    }

    #[inline]
    pub fn health_check_timeout(&self) -> Option<Duration> {
        self.health_check_timeout
            .or(self.all_timeout)
            .map(|x| x as u64)
            .map(Duration::from_millis)
    }
}

/// Retry conditions of a service.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Retry {
    /// Maximum attempts to retry executing
    #[serde(default)]
    pub max_attempts: i32,

    /// Delay time of retrying the service, in milliseconds
    #[serde(default)]
    pub delay: u64,
}

/// Watchdog configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Watchdog {
    /// Kind of the watchdog to use.
    pub kind: Option<WatchdogKind>,

    /// Time interval of watchdog checking.
    #[serde(default = "Watchdog::default_interval")]
    pub health_interval: u32,

    /// Also mark the service failed on successful exits (`$? == 0`)
    #[serde(default)]
    pub successful_exit: bool,
}
impl Watchdog {
    fn default_interval() -> u32 {
        5000
    }
}

/// Kind of service watchdog.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum WatchdogKind {
    /// The supervisor polls to execute the health check command.
    HealthCheck,

    /// The service regularly notifies the supervisor that it's normally running.
    Notify,
}

/// Resource limitation.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Reslimit {
    /// Max CPU usage.
    pub cpu: Option<u64>,

    /// Max memory usage.
    pub memory: Option<u64>,
}
