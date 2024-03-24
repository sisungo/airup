//! Logger interface of the Airup supervisor.

#[cfg(feature = "fallback_logger")]
pub mod fallback_logger;

use airup_sdk::system::LogRecord;
use anyhow::anyhow;

/// Represents to a client of a logger system.
#[async_trait::async_trait]
pub trait Logger: Send + Sync {
    /// Writes an log record to the logger.
    async fn write(&mut self, subject: &str, module: &str, msg: &[u8]) -> anyhow::Result<()>;

    /// Queries last `n` records of specified subject.
    async fn tail(&mut self, subject: &str, n: usize) -> anyhow::Result<Vec<LogRecord>>;
}

/// A wrapper around [`Logger`] to provide management.
pub struct Manager {
    inner: tokio::sync::Mutex<Box<dyn Logger>>,
}
impl Manager {
    /// Creates a new [`Manager`] instance.
    pub fn new() -> Self {
        Self {
            inner: tokio::sync::Mutex::new(Box::new(NulLogger)),
        }
    }

    /// Sets a new [`Logger`] instance.
    pub async fn set_logger(&self, new: Box<dyn Logger>) -> Box<dyn Logger> {
        std::mem::replace(&mut *self.inner.lock().await, new)
    }

    /// Sets a new [`Logger`] instance by name.
    #[allow(dead_code, unused)]
    pub async fn set_logger_by_name(
        &self,
        name: &str,
    ) -> Result<Box<dyn Logger>, airup_sdk::Error> {
        let name = name.strip_suffix(".airx").unwrap_or(name);

        #[cfg(feature = "fallback_logger")]
        if name == "fallback_logger" {
            return Ok(self
                .set_logger(Box::new(fallback_logger::FallbackLogger::new()))
                .await);
        }

        Err(airup_sdk::Error::NotFound)
    }

    /// Removes the [`Logger`] instance which was previously set.
    pub async fn remove_logger(&self) {
        self.set_logger(Box::new(NulLogger)).await;
    }

    /// Invokes [`Logger::write`] on the inner logger.
    pub async fn write(&self, subject: &str, module: &str, msg: &[u8]) -> anyhow::Result<()> {
        self.inner.lock().await.write(subject, module, msg).await
    }

    /// Invokes [`Logger::tail`] on the inner logger.
    pub async fn tail(&self, subject: &str, n: usize) -> anyhow::Result<Vec<LogRecord>> {
        self.inner.lock().await.tail(subject, n).await
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
    async fn write(&mut self, _: &str, _: &str, _: &[u8]) -> anyhow::Result<()> {
        Err(anyhow!("no available loggers"))
    }

    async fn tail(&mut self, _: &str, _: usize) -> anyhow::Result<Vec<LogRecord>> {
        Err(anyhow!("no available loggers"))
    }
}
