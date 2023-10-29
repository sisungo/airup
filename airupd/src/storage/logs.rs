//! Represents to Airup's logs directory.

use airupfx::prelude::*;
use std::path::Path;

/// Main navigator of Airup's logs directory.
#[derive(Debug)]
pub struct Logs {
    base_chain: DirChain<'static>,
}
impl Logs {
    /// Creates a new [Logs] instance.
    pub fn new() -> Self {
        Self {
            base_chain: DirChain::new(airupfx::config::build_manifest().log_dir),
        }
    }

    /// Attempts to open a log file in appending mode.
    ///
    /// If the file does not exist, it will be created.
    pub async fn open_append<P: AsRef<Path>>(&self, path: P) -> std::io::Result<tokio::fs::File> {
        let path = self
            .base_chain
            .find(path)
            .await
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;
        tokio::fs::File::options()
            .create(true)
            .append(true)
            .open(path)
            .await
    }

    /// Attempts to open a log file for reading.
    pub async fn open_read<P: AsRef<Path>>(&self, path: P) -> std::io::Result<tokio::fs::File> {
        let path = self
            .base_chain
            .find(path)
            .await
            .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;
        tokio::fs::File::options().read(true).open(path).await
    }
}
impl Default for Logs {
    fn default() -> Self {
        Self::new()
    }
}
