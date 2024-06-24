//! Airupd-flavored presets for the [`tracing`] framework.

use tracing::metadata::LevelFilter;
use tracing_subscriber::{filter::filter_fn, prelude::*};

/// Builder of `airupd`-flavor tracing configuration.
#[derive(Debug, Clone)]
pub struct Builder {
    name: String,
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
        let verbose = self.verbose;
        let level_filter = match self.quiet {
            true => LevelFilter::ERROR,
            false => match verbose {
                true => LevelFilter::TRACE,
                false => LevelFilter::INFO,
            },
        };

        let stdio_layer = tracing_subscriber::fmt::layer()
            .without_time()
            .with_ansi(self.color)
            .with_file(false)
            .with_writer(std::io::stderr)
            .with_target(false)
            .with_filter(filter_fn(move |metadata| {
                verbose || metadata.target().contains("console")
            }))
            .with_filter(level_filter);

        tracing_subscriber::registry().with(stdio_layer).init();
    }
}
impl Default for Builder {
    fn default() -> Self {
        Self {
            name: "airupd".into(),
            quiet: false,
            verbose: false,
            color: true,
        }
    }
}
