#![allow(unused)]

use super::Security;
use ahash::HashMap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{path::Path, sync::OnceLock};

pub static MANIFEST: OnceLock<BuildManifest> = OnceLock::new();

/// Represents to `build_manifest.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildManifest {
    /// Name of the running operating system.
    #[serde(default = "default_os_name")]
    pub os_name: &'static str,

    /// Path of Airup's system-wide config directory, e.g. `/etc/airup`.
    pub config_dir: &'static Path,

    /// Path of Airup's system-wide service directory, e.g. `/etc/airup/services`.
    pub service_dir: &'static Path,

    /// Path of Airup's system-wide milestone directory, e.g. `/etc/airup/milestones`.
    pub milestone_dir: &'static Path,

    /// Path of Airup's system-wide runtime directory, e.g. `/run/airup`.
    pub runtime_dir: &'static Path,

    /// Path of Airup's system-wide log directory, e.g. `/var/log/airup`.
    pub log_dir: &'static Path,

    /// Table of initial environment variables.
    #[serde(default)]
    pub env_vars: HashMap<&'static str, Option<&'static str>>,

    /// Commands executed in `early_boot` pseudo-milestone.
    #[serde(default)]
    pub early_cmds: Vec<&'static str>,

    /// Default security model to use.
    #[serde(default)]
    pub security: Security,
}

fn default_os_name() -> &'static str {
    "\x1b[36;4mAirup\x1b[0m"
}

pub fn manifest() -> &'static BuildManifest {
    MANIFEST.get_or_init(|| {
        serde_json::from_str(include_str!("../../../build_manifest.json")).expect("bad airup build")
    })
}

/// Sets the build manifest to the specific value.
///
/// ## Panic
/// Panics if the manifest is already set, which may be done by any call of [manifest] or [set_manifest].
pub fn set_manifest(manifest: BuildManifest) {
    MANIFEST.set(manifest);
}
