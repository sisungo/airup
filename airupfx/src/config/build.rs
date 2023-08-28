use crate::config::SystemConf;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::OnceLock};

static MANIFEST: OnceLock<Manifest> = OnceLock::new();

/// Represents to `build_manifest.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Path of Airup's system-wide config directory, e.g. `/etc/airup`.
    pub config_dir: PathBuf,

    /// Path of Airup's system-wide service directory, e.g. `/etc/airup/services`.
    pub service_dir: PathBuf,

    /// Path of Airup's system-wide milestone directory, e.g. `/etc/airup/milestones`.
    pub milestone_dir: PathBuf,

    /// Path of Airup's system-wide runtime directory, e.g. `/run/airup`.
    pub runtime_dir: PathBuf,

    /// Default content of Airup's system-wide config which is used when `$config_dir/system.conf` doesn't exist.
    pub default_system_conf: SystemConf,
}
impl Manifest {
    /// Initializes the global `Manifest` instance for use of [manifest].
    #[inline]
    pub fn init() {
        let manifest = serde_json::from_slice(include_bytes!("../../../build_manifest.json"))
            .expect("bad `build_manifest.json`");

        MANIFEST.set(manifest).unwrap();
    }
}

/// Returns a reference to the unique [Manifest].
///
/// ## Panic
/// Panics if `Manifest::init()` hasn't been called.
#[inline]
pub fn manifest() -> &'static Manifest {
    MANIFEST.get().unwrap()
}
