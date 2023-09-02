mod build;
mod system_conf;

pub use build::Security;
pub use system_conf::SystemConf;

use build::BuildManifest;
use std::sync::OnceLock;

static SYSTEM_CONF: OnceLock<SystemConf> = OnceLock::new();

/// Initializes the main configuration
#[inline]
pub async fn init() {
    let system_conf = SystemConf::new().await;
    system_conf.env.override_env();
    SYSTEM_CONF.set(system_conf).unwrap();
}

/// Returns a reference to the global unique [SystemConf] instance.
///
/// ## Panic
/// Panics if [init] hasn't been called.
#[inline]
pub fn system_conf() -> &'static SystemConf {
    SYSTEM_CONF.get().unwrap()
}

/// Returns a reference to the unique [Manifest].
///
/// ## Panic
/// Panics if `Manifest::init()` hasn't been called.
#[inline]
pub fn build_manifest() -> &'static BuildManifest {
    static MANIFEST: OnceLock<BuildManifest> = OnceLock::new();

    MANIFEST.get_or_init(|| include!(concat!(env!("OUT_DIR"), "/build_manifest.rs")))
}
