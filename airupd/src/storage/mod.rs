//! Storage subsystem of the Airup daemon.

mod config;
mod milestones;
mod runtime;
mod services;

use self::config::Config;
use self::milestones::Milestones;
use self::runtime::Runtime;
use self::services::Services;
use airup_sdk::files::{ReadError, Service};

/// Main navigator of Airup's storage.
#[derive(Debug)]
pub struct Storage {
    pub config: Config,
    pub runtime: Runtime,
    pub services: Services,
    pub milestones: Milestones,
}
impl Storage {
    /// Creates a new [`Storage`] instance.
    pub async fn new() -> Self {
        Self {
            config: Config::new().await,
            runtime: Runtime::new().await,
            services: Services::new(),
            milestones: Milestones::new(),
        }
    }

    /// Gets a patched service installation.
    pub async fn get_service_patched(&self, name: &str) -> Result<Service, ReadError> {
        let patch = self.config.of_service(name).await;
        self.services.get_and_patch(name, patch).await
    }
}
