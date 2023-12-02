//! Represents to Airup's system config.

use ahash::HashMap;
use airupfx::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct Config {
    pub base_dir: DirChain<'static>,
    pub system_conf: SystemConf,
}
impl Config {
    /// Creates a new [`Config`] instance.
    pub async fn new() -> Self {
        let base_dir = airup_sdk::build::manifest().config_dir.clone();
        let system_conf = SystemConf::read_from(&base_dir.join("system.airc"))
            .await
            .unwrap_or_default();

        Self {
            base_dir: base_dir.into(),
            system_conf,
        }
    }

    /// Returns path of separated config file for specified service.
    pub async fn of_service(&self, name: &str) -> Option<PathBuf> {
        self.base_dir.find(format!("{name}.service.airc")).await
    }
}

/// Representation of Airup's system config.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SystemConf {
    #[serde(default)]
    pub system: System,

    #[serde(default)]
    pub env: Env,
}
impl SystemConf {
    /// Parses TOML format [`SystemConf`] from given `path`.
    async fn read_from(path: &Path) -> anyhow::Result<Self> {
        let s = tokio::fs::read_to_string(path).await?;
        Ok(toml::from_str(&s)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    #[serde(default = "default_os_name")]
    pub os_name: String,
}
impl Default for System {
    fn default() -> Self {
        Self {
            os_name: default_os_name(),
        }
    }
}

/// Represents to Airup's environment.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Env {
    /// Table of initial environment variables.
    ///
    /// If a value is set to `null`, the environment variable gets removed if it exists.
    #[serde(default)]
    vars: HashMap<String, Option<String>>,
}
impl Env {
    /// Overrides the environment with this [`Env`] object.
    pub fn override_env(&self) {
        let mut vars: BTreeMap<String, Option<String>> = BTreeMap::new();
        for (k, v) in &airup_sdk::build::manifest().env_vars {
            vars.insert(k.to_owned(), v.as_ref().map(Into::into));
        }
        for (k, v) in &self.vars {
            vars.insert(k.into(), v.clone());
        }
        airupfx::env::set_vars(vars);
    }
}

fn default_os_name() -> String {
    airup_sdk::build::manifest().os_name.clone()
}
