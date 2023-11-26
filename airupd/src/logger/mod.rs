//! Logger interface of the Airup supervisor.

use anyhow::anyhow;

/// Represents to a client of a logger system.
#[async_trait::async_trait]
pub trait Logger {
    /// Writes an log record to the logger.
    async fn write(&mut self) -> anyhow::Result<()>;
}

/// A wrapper around [`Logger`] to provide management.
pub struct Manager {
    inner: tokio::sync::RwLock<Box<dyn Logger + Send + Sync>>,
}
impl Manager {
    /// Creates a new [`Manager`] instance.
    pub fn new() -> Self {
        Self {
            inner: tokio::sync::RwLock::new(Box::new(NulLogger)),
        }
    }

    /// Sets a new [`Logger`] instance.
    pub async fn set_logger(
        &self,
        new: Box<dyn Logger + Send + Sync>,
    ) -> Box<dyn Logger + Send + Sync> {
        std::mem::replace(&mut *self.inner.write().await, new)
    }

    /// Removes the [`Logger`] instance which was previously set.
    pub async fn remove_logger(&self) {
        self.set_logger(Box::new(NulLogger)).await;
    }

    /// Invokes [`Logger::write`] on the inner logger.
    pub async fn write(&self) -> anyhow::Result<()> {
        self.inner.write().await.write().await
    }
}
impl std::fmt::Debug for Manager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Manager")
    }
}
impl Default for Manager {
    fn default() -> Self {
        Self::new()
    }
}

/// A [`Logger`] implementation that does not log at all.
struct NulLogger;
#[async_trait::async_trait]
impl Logger for NulLogger {
    async fn write(&mut self) -> anyhow::Result<()> {
        Err(anyhow!("no such logger"))
    }
}
