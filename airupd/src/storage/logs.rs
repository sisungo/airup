//! Represents to Airup's logs directory.

use std::path::{Path, PathBuf};

/// Main navigator of Airup's logs directory.
#[derive(Debug)]
pub struct Logs {
    base_dir: PathBuf,
}
impl Logs {
    /// Creates a new [`Logs`] instance.
    pub fn new() -> Self {
        Self {
            base_dir: airup_sdk::build::manifest().log_dir.clone(),
        }
    }

    /// Attempts to open a log file in appending mode.
    ///
    /// If the file does not exist, it will be created.
    pub async fn open_append<P: AsRef<Path>>(&self, path: P) -> std::io::Result<tokio::fs::File> {
        let path = self.base_dir.join(path);
        tokio::fs::File::options()
            .create(true)
            .append(true)
            .open(path)
            .await
    }

    /// Attempts to open a log file for reading.
    pub async fn open_read<P: AsRef<Path>>(&self, path: P) -> std::io::Result<tokio::fs::File> {
        let path = self.base_dir.join(path);
        tokio::fs::File::options().read(true).open(path).await
    }
}
impl Default for Logs {
    fn default() -> Self {
        Self::new()
    }
}
