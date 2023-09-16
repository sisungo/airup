//! Represents to Airup's system config.

use super::Security;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::OnceLock,
};

static SYSTEM_CONF: OnceLock<SystemConf> = OnceLock::new();

/// Representation of Airup's system config.
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
    /// Creates a new [SystemConf] instance. This firstly reads from `$config_dir/system.conf`, or returns the default value
    /// if fails.
    pub async fn new() -> Self {
        Self::read_from(&super::build_manifest().config_dir.join("system.conf"))
            .await
            .unwrap_or_default()
    }

    /// Parses TOML format [SystemConf] from given `path`.
    async fn read_from(path: &Path) -> anyhow::Result<Self> {
        let s = tokio::fs::read_to_string(path).await?;
        Ok(toml::from_str(&s)?)
    }

    pub async fn init() {
        let obj = SystemConf::new().await;
        obj.env.override_env();
        SYSTEM_CONF.set(obj).unwrap();
    }

    /// Returns a reference to the global unique [SystemConf] instance.
    ///
    /// ## Panic
    /// Panics if the instance has not be initialized yet.
    pub fn get() -> &'static SystemConf {
        SYSTEM_CONF.get().unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct System {
    #[serde(default = "default_os_name")]
    pub os_name: Cow<'static, str>,

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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Env {
    /// Table of initial environment variables.
    ///
    /// If a value is set to `null`, the environment variable gets removed if it exists.
    #[serde(default)]
    vars: BTreeMap<String, Option<String>>,
}
impl Env {
    /// Overrides the environment with this [Env] object.
    fn override_env(&self) {
        let mut vars: BTreeMap<String, Option<String>> = BTreeMap::new();
        for (k, v) in super::build_manifest().env_vars {
            vars.insert(k.to_string(), v.map(Into::into));
        }
        for (k, v) in self.vars.iter() {
            vars.insert(k.into(), v.clone());
        }
        crate::env::set_vars(vars);
    }
}

fn default_os_name() -> Cow<'static, str> {
    super::build_manifest().os_name.into()
}

fn default_security() -> Security {
    super::build_manifest().security.clone()
}
