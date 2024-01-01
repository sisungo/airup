use ahash::HashMap;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Representation of Airup's system config.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SystemConf {
    #[serde(default)]
    pub system: System,

    #[serde(default)]
    pub env: Env,
}
impl SystemConf {
    /// Parses TOML format [`SystemConf`] from given `path`.
    pub async fn read_from<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        Self::_read_from(path.as_ref()).await
    }

    async fn _read_from(path: &Path) -> anyhow::Result<Self> {
        let s = tokio::fs::read_to_string(path).await?;
        Ok(toml::from_str(&s)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct System {
    #[serde(default = "default_os_name")]
    pub os_name: String,

    #[serde(default = "default_reboot_timeout")]
    pub reboot_timeout: u32,
}
impl Default for System {
    fn default() -> Self {
        Self {
            os_name: default_os_name(),
            reboot_timeout: default_reboot_timeout(),
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
    pub vars: HashMap<String, Option<String>>,
}

fn default_os_name() -> String {
    crate::build::manifest().os_name.clone()
}

fn default_reboot_timeout() -> u32 {
    5000
}
