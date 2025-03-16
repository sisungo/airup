use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Representation of Airup's system config.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SystemConf {
    #[serde(default)]
    pub system: System,

    #[serde(default)]
    pub env: Env,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct System {
    #[serde(default = "default_os_name")]
    pub os_name: String,

    #[serde(default = "default_reboot_timeout")]
    pub reboot_timeout: u32,

    #[serde(default)]
    pub instance_name: String,
}
impl Default for System {
    fn default() -> Self {
        Self {
            os_name: default_os_name(),
            reboot_timeout: default_reboot_timeout(),
            instance_name: String::new(),
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
    pub vars: HashMap<String, toml::Value>,
}

fn default_os_name() -> String {
    crate::build::manifest().os_name.clone()
}

fn default_reboot_timeout() -> u32 {
    1200000
}
