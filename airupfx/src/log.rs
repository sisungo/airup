//! The Airup Logger
//! Airup uses [tracing] as its logging framework.

use std::path::PathBuf;
use tracing::metadata::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{filter::filter_fn, fmt::writer::OptionalWriter, prelude::*};

/// Builder of AirupFX's-flavor tracing.
#[derive(Debug, Clone)]
pub struct Builder {
    name: String,
    quiet: bool,
    color: bool,
    location: Option<PathBuf>,
}
impl Builder {
    /// Creates a new `Builder` with default settings.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the logger's name.
    #[inline]
    pub fn name<S: Into<String>>(&mut self, s: S) -> &mut Self {
        self.name = s.into();
        self
    }

    /// Sets whether console output is disabled for the logger.
    #[inline]
    pub fn quiet(&mut self, val: bool) -> &mut Self {
        self.quiet = val;
        self
    }

    /// Sets whether colorful console output is enabled for the logger.
    #[inline]
    pub fn color(&mut self, val: bool) -> &mut Self {
        self.color = val;
        self
    }

    /// Sets the location where the logger stores log files.
    #[inline]
    pub fn location(&mut self, val: Option<PathBuf>) -> &mut Self {
        self.location = val;
        self
    }

    /// Initializes the logger.
    #[inline]
    pub fn init(&mut self) -> (WorkerGuard, WorkerGuard) {
        let stdio_appender: OptionalWriter<_> = match self.quiet {
            true => OptionalWriter::none(),
            false => OptionalWriter::some(std::io::stderr()),
        };
        let file_appender: OptionalWriter<_> = self
            .location
            .as_ref()
            .map(|path| tracing_appender::rolling::daily(path, format!("{}.log", self.name)))
            .into();

        let (stdio_appender, stdio_guard) = tracing_appender::non_blocking(stdio_appender);
        let (file_appender, file_guard) = tracing_appender::non_blocking(file_appender);

        let stdio_layer = tracing_subscriber::fmt::layer()
            .without_time()
            .with_ansi(self.color)
            .with_file(false)
            .with_writer(stdio_appender)
            .with_target(false)
            .with_filter(filter_fn(|metadata| metadata.target().contains("console")))
            .with_filter(LevelFilter::INFO);
        let file_layer = tracing_subscriber::fmt::layer()
            .with_ansi(false)
            .with_writer(file_appender)
            .with_filter(
                crate::env::take_var("AIRUP_LOG")
                    .as_deref()
                    .unwrap_or("info")
                    .parse::<LevelFilter>()
                    .unwrap_or(LevelFilter::INFO),
            );

        tracing_subscriber::registry()
            .with(stdio_layer)
            .with(file_layer)
            .init();

        (stdio_guard, file_guard)
    }
}
impl Default for Builder {
    fn default() -> Self {
        Self {
            name: "airupfx".into(),
            quiet: false,
            color: true,
            location: None,
        }
    }
}

/// An extension for standard [Result] type to support logging.
#[cfg(feature = "process")]
pub trait ResultExt<T> {
    /// Returns the contained `Ok` value, consuming the `self` value.
    fn unwrap_log(self, why: &str) -> T;
}
#[cfg(feature = "process")]
impl<T, E> ResultExt<T> for Result<T, E>
where
    E: std::fmt::Display,
{
    fn unwrap_log(self, why: &str) -> T {
        match self {
            Ok(val) => val,
            Err(err) => {
                tracing::error!(target: "console", "{}: {}", why, err);
                crate::process::emergency();
            }
        }
    }
}

/// An extension for standard [Option] type to support logging.
#[cfg(feature = "process")]
pub trait OptionExt<T> {
    /// Returns the contained `Ok` value, consuming the `self` value.
    fn unwrap_log(self, why: &str) -> T;
}
#[cfg(feature = "process")]
impl<T> OptionExt<T> for Option<T> {
    fn unwrap_log(self, why: &str) -> T {
        match self {
            Some(val) => val,
            None => {
                tracing::error!(target: "console", "{why}");
                crate::process::emergency();
            }
        }
    }
}
