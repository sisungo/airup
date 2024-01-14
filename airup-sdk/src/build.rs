//! Information about an Airup build.

use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::OnceLock};

static MANIFEST: OnceLock<BuildManifest> = OnceLock::new();

/// Represents to the structure of the build manifest, which is usually read from `build_manifest.json` at compile-time.
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

    /// Path of Airup's system-wide log directory, e.g. `/var/log/airup`.
    pub log_dir: PathBuf,

    /// Table of initial environment variables.
    #[serde(default)]
    pub env_vars: ahash::HashMap<String, Option<String>>,

    /// Commands executed in `early_boot` pseudo-milestone.
    #[serde(default)]
    pub early_cmds: Vec<String>,
}

fn default_os_name() -> String {
    "\x1b[36;4mAirup\x1b[0m".into()
}

/// Gets a reference to the global [`BuildManifest`] instance. If [`set_manifest`] was not previously called, it automatically
/// initializes the instance by reading the compile-time `build_manifest.json`.
///
/// # Panics
/// Panics if the [`BuildManifest`] instance was not initialized yet and the compile-time `build_manifest.json` was invalid.
pub fn manifest() -> &'static BuildManifest {
    #[cfg(feature = "_internal")]
    {
        MANIFEST.get_or_init(embedded_manifest)
    }

    #[cfg(not(feature = "_internal"))]
    {
        MANIFEST.get().unwrap()
    }
}

/// Sets the build manifest to the specific value.
///
/// # Panics
/// Panics if the manifest is already set, which may be done by any call of [`manifest`] or [`set_manifest`].
pub fn set_manifest(manifest: BuildManifest) {
    MANIFEST.set(manifest).unwrap();
}

/// Returns the embedded [`BuildManifest`] instance.
///
/// # Panics
/// Panics if the compile-time `build_manifest.json` was invalid.
#[doc(hidden)]
#[cfg(feature = "_internal")]
pub fn embedded_manifest() -> BuildManifest {
    serde_json::from_str(include_str!("../../build_manifest.json")).expect("bad airup build")
}
