//! Information about an Airup build.

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::OnceLock};

static MANIFEST: OnceLock<BuildManifest> = OnceLock::new();

/// Represents to the structure of the build manifest, which is usually read from `build_manifest.json` at compile-time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildManifest {
    /// Name of the running operating system.
    #[serde(default = "default_os_name")]
    pub os_name: String,

    /// Path of Airup's config directory, e.g. `/etc/airup`.
    pub config_dir: PathBuf,

    /// Path of Airup's service directory, e.g. `/etc/airup/services`.
    pub service_dir: PathBuf,

    /// Path of Airup's milestone directory, e.g. `/etc/airup/milestones`.
    pub milestone_dir: PathBuf,

    /// Path of Airup's runtime directory, e.g. `/run/airup`.
    pub runtime_dir: PathBuf,

    /// Table of initial environment variables.
    #[serde(default)]
    pub env_vars: HashMap<String, Option<String>>,

    /// Commands executed in `early_boot` pseudo-milestone.
    #[serde(default)]
    pub early_cmds: Vec<String>,

    /// Name of Airup's socket in the abstract namespace. This is Linux-only.
    #[cfg(target_os = "linux")]
    pub linux_ipc_name: Option<String>,
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

/// Sets the build manifest to the specific value.
///
/// # Panics
/// Panics if the manifest is already set, which may be done by any call of [`manifest`] or [`set_manifest`].
pub fn try_set_manifest(manifest: BuildManifest) -> Option<()> {
    MANIFEST.set(manifest).ok()
}

/// Returns the embedded [`BuildManifest`] instance.
///
/// # Panics
/// Panics if the compile-time `build_manifest.json` was invalid.
#[doc(hidden)]
#[cfg(feature = "_internal")]
fn embedded_manifest() -> BuildManifest {
    ciborium::from_reader(&include_bytes!(concat!(env!("OUT_DIR"), "/build_manifest.cbor"))[..])
        .expect("bad airup build")
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "_internal")]
    #[test]
    fn embedded_manifest() {
        super::embedded_manifest();
    }
}
