//! # Airup Storage

mod config;
mod runtime;

use self::config::Config;
use self::runtime::Runtime;

/// Main navigator of Airup's storage.
#[derive(Debug)]
pub struct Storage {
    pub config: Config,
    pub runtime: Runtime,
}
impl Storage {
    /// Creates a new [Storage] instance.
    #[inline]
    pub async fn new() -> Self {
        Self {
            config: Config::new(),
            runtime: Runtime::new().await,
        }
    }
}
