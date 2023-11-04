mod build;
mod system_conf;

pub use build::{manifest as build_manifest, set_manifest as set_build_manifest};
pub use system_conf::SystemConf;

/// Initializes the main configuration
#[inline]
pub async fn init() {
    SystemConf::init().await;
}

/// Returns a reference to the global unique [`SystemConf`] instance.
///
/// ## Panic
/// Panics if [init] hasn't been called.
#[inline]
#[must_use]
pub fn system_conf() -> &'static SystemConf {
    SystemConf::get()
}
