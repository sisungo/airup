//! # Airup Storage

mod config;
mod milestones;
mod runtime;
mod services;

use self::config::Config;
use self::milestones::Milestones;
use self::runtime::Runtime;
use self::services::Services;

/// Main navigator of Airup's storage.
#[derive(Debug)]
pub struct Storage {
    pub config: Config,
    pub runtime: Runtime,
    pub services: Services,
    pub milestones: Milestones,
}
impl Storage {
    /// Creates a new [Storage] instance.
    #[inline]
    pub async fn new() -> Self {
        Self {
            config: Config::new(),
            runtime: Runtime::new().await,
            services: Services::new(),
            milestones: Milestones::new(),
        }
    }
}
