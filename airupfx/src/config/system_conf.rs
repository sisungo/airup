//! Represents to Airup's system config.

use super::Security;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Represents to Airup's system config.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SystemConf {
    #[serde(default)]
    pub system: System,

    #[serde(default)]
    pub locations: Locations,

    #[serde(default)]
    pub env: Env,
}
impl SystemConf {
    /// Creates a new `SystemConf` instance. This firstly reads from `$config_dir/system.conf`, or returns the default value
    /// if fails.
    pub async fn new() -> Self {
        Self::read_from(&super::build_manifest().config_dir.join("system.conf"))
            .await
            .unwrap_or_default()
    }

    /// Parses TOML format `SystemConf` from `path`.
    async fn read_from(path: &Path) -> anyhow::Result<Self> {
        let s = tokio::fs::read_to_string(path).await?;
        Ok(toml::from_str(&s)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    #[serde(default = "default_os_name")]
    pub os_name: String,

    #[serde(default = "default_security")]
    pub security: Security,
}
impl Default for System {
    fn default() -> Self {
        Self {
            os_name: default_os_name(),
            security: default_security(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Locations {
    pub logs: Option<PathBuf>,
}

/// Represents to Airup's environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Env {
    /// Table of initial environment variables.
    ///
    /// If a value is set to `null`, the environment variable gets removed if it exists.
    #[serde(default = "default_env_vars")]
    pub vars: BTreeMap<String, Option<String>>,
}
impl Env {
    /// Overrides the environment with this [Env] object.
    pub fn override_env(&self) {
        crate::env::set_vars(self.vars.clone());
    }
}
impl Default for Env {
    fn default() -> Self {
        Self {
            vars: default_env_vars(),
        }
    }
}

fn default_env_vars() -> BTreeMap<String, Option<String>> {
    super::build_manifest().env_vars.clone()
}

fn default_os_name() -> String {
    super::build_manifest().os_name.clone()
}

fn default_security() -> Security {
    super::build_manifest().security.clone()
}
