mod build;
mod system_conf;

pub use system_conf::SystemConf;

use build::BuildManifest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Security {
    /// Additional security checks are disabled
    Disabled,

    /// Use a simple security checker which allows access from `root` only
    Simple,

    /// Use Airup security policy
    #[default]
    Policy,
}

/// Initializes the main configuration
#[inline]
pub async fn init() {
    SystemConf::init().await;
}

/// Returns a reference to the global unique [SystemConf] instance.
///
/// ## Panic
/// Panics if [init] hasn't been called.
#[inline]
pub fn system_conf() -> &'static SystemConf {
    SystemConf::get()
}

/// Returns a reference to the unique [BuildManifest].
#[inline]
pub fn build_manifest() -> &'static BuildManifest {
    BuildManifest::get()
}
