//! Represents to Airup's config directory.

use airupfx::{policy, prelude::*};

/// Main navigator of Airup's config directory.
#[derive(Debug)]
pub struct Config {
    pub policy: policy::Db,
}
impl Config {
    /// Creates a new [Config] instance.
    pub async fn new() -> Self {
        let base_dir = &airupfx::config::build_manifest().config_dir;

        Self {
            policy: policy::Db::new(DirChain::from(base_dir.join("policy"))).await,
        }
    }
}
