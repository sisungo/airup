//! Represents to Airup's config directory.

use airupfx::{policy, prelude::*};

/// Main navigator of Airup's config directory.
#[derive(Debug)]
pub struct Config {
    pub policy: ConcurrentInit<policy::Db>,
}
impl Config {
    /// Creates a new [Config] instance.
    #[inline]
    pub fn new() -> Self {
        let base_dir = &airupfx::config::build_manifest().config_dir;

        Self {
            policy: ConcurrentInit::new(policy::Db::new(DirChain::from(base_dir.join("policy")))),
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
