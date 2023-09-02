use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

/// Represents to `build_manifest.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildManifest {
    /// Name of the running operating system.
    #[serde(default = "default_os_name")]
    pub os_name: String,

    /// Path of Airup's system-wide config directory, e.g. `/etc/airup`.
    pub config_dir: PathBuf,

    /// Path of Airup's system-wide service directory, e.g. `/etc/airup/services`.
    pub service_dir: PathBuf,

    /// Path of Airup's system-wide milestone directory, e.g. `/etc/airup/milestones`.
    pub milestone_dir: PathBuf,

    /// Path of Airup's system-wide runtime directory, e.g. `/run/airup`.
    pub runtime_dir: PathBuf,

    /// Table of initial environment variables.
    #[serde(default)]
    pub env_vars: BTreeMap<String, Option<String>>,

    #[serde(default)]
    pub security: Security,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Security {
    #[serde(alias = "disabled")]
    Disabled,

    #[serde(alias = "simple")]
    Simple,

    #[default]
    #[serde(alias = "policy")]
    Policy,
}

fn default_os_name() -> String {
    "\x1b[36;4mAirup\x1b[0m".into()
}
