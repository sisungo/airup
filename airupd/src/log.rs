//! Airupd logging facility.

use std::sync::OnceLock;
use tokio::{sync::mpsc, task::AbortHandle};

static LOGGER: OnceLock<Logger> = OnceLock::new();

#[macro_export]
macro_rules! inform {
    ($($arg:tt)*) => {
        $crate::log::logger().inform(format!($($arg)*)).await
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        $crate::log::logger().warn(format!($($arg)*)).await
    };
}

#[macro_export]
macro_rules! report_error {
    ($($arg:tt)*) => {
        $crate::log::logger().report_error(format!($($arg)*)).await
    };
}

/// Builder of `airupd`-flavor tracing configuration.
#[derive(Debug, Clone)]
pub struct Builder {
    quiet: bool,
    verbose: bool,
    color: bool,
}
impl Builder {
    /// Creates a new [`Builder`] instance with default settings.
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether console output is disabled for the logger.
    #[inline]
    pub fn quiet(&mut self, val: bool) -> &mut Self {
        self.quiet = val;
        self
    }

    /// Sets whether console output is verbose for the logger.
    #[inline]
    pub fn verbose(&mut self, val: bool) -> &mut Self {
        self.verbose = val;
        self
    }

    /// Sets whether colorful console output is enabled for the logger.
    #[inline]
    pub fn color(&mut self, val: bool) -> &mut Self {
        self.color = val;
        self
    }

    /// Initializes the logger.
    #[inline]
    pub fn init(&mut self) {
        let (tx, rx) = mpsc::channel(16);
        let logger_impl = LoggerImpl { rx };
        let background = tokio::spawn(logger_impl.run()).abort_handle();
        LOGGER.set(Logger { tx, background }).unwrap();
    }
}
impl Default for Builder {
    fn default() -> Self {
        Self {
            quiet: false,
            verbose: false,
            color: true,
        }
    }
}

#[derive(Debug)]
pub struct Logger {
    tx: mpsc::Sender<Request>,
    background: AbortHandle,
}
impl Logger {
    pub async fn inform(&self, s: String) {
        _ = self.tx.send(Request::Inform(s)).await;
    }

    pub async fn warn(&self, s: String) {
        _ = self.tx.send(Request::Warn(s)).await;
    }

    pub async fn report_error(&self, s: String) {
        _ = self.tx.send(Request::ReportError(s)).await;
    }
}
impl Drop for Logger {
    fn drop(&mut self) {
        self.background.abort();
    }
}

pub fn logger() -> &'static Logger {
    LOGGER.get().unwrap()
}

struct LoggerImpl {
    rx: mpsc::Receiver<Request>,
}
impl LoggerImpl {
    async fn run(mut self) {
        while let Some(req) = self.rx.recv().await {
            match req {
                Request::Inform(s) => {
                    eprintln!("\x1b[32m * \x1b[0m {s}");
                }
                Request::Warn(s) => {
                    eprintln!("\x1b[33m * \x1b[0m {s}");
                }
                Request::ReportError(s) => {
                    eprintln!("\x1b[31m * \x1b[0m {s}");
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
enum Request {
    Inform(String),
    Warn(String),
    ReportError(String),
}
