//! # Airup Storage

mod config;
mod logs;
mod milestones;
mod runtime;
mod services;

use self::config::Config;
use self::logs::Logs;
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
    pub logs: Logs,
}
impl Storage {
    /// Creates a new [Storage] instance.
    pub async fn new() -> Self {
        Self {
            config: Config::new().await,
            runtime: Runtime::new().await,
            services: Services::new(),
            milestones: Milestones::new(),
            logs: Logs::new(),
        }
    }
}
