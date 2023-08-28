//! Represents to Airup's system config.

use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

/// Represents to Airup's system config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConf {
    #[serde(default)]
    pub system: System,

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
impl Default for SystemConf {
    fn default() -> Self {
        super::build_manifest().default_system_conf.clone()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct System {
    os_name: Option<String>,
}
impl System {
    pub fn os_name(&self) -> &str {
        self.os_name.as_deref().unwrap_or("\x1b[36;4mAirup\x1b[0m")
    }
}

/// Represents to Airup's environment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Env {
    /// Environment variables to execute for the service.
    ///
    /// If a value is set to `null`, the environment variable gets removed if it exists.
    #[serde(default)]
    pub vars: BTreeMap<String, Option<String>>,
}
impl Env {
    /// Overrides the environment with this [Env] object.
    pub fn override_env(&self) {
        crate::env::set_vars(self.vars.clone());
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Locations {
    pub logs: Option<PathBuf>,
}
