//! Inspection and manipulation of `airupd`’s environment.

use std::borrow::Cow;
use std::sync::OnceLock;

#[derive(Debug, Clone)]
pub struct Cmdline {
    /// Enable Airup verbose console outputs
    pub verbose: bool,

    /// Disable Airupd console outputs
    pub quiet: bool,

    /// Disable colors for Airupd console outputs
    pub no_color: bool,

    /// Specify Airup milestone
    pub milestone: Cow<'static, str>,
}
impl Cmdline {
    /// Parses a new [`Cmdline`] instance from the command-line arguments. This function will automatically detect the
    /// environment to detect the style of the parser.
    pub fn parse() -> Self {
        Self::parse_linux_init()
    }

    /// A command-line argument parser that assumes arguments are Linux-init styled.
    fn parse_linux_init() -> Self {
        let mut object = Self::default();

        for arg in std::env::args() {
            if arg == "single" {
                object.milestone = "single-user".into();
            }
        }

        if let Ok(x) = airupfx::env::take_var("AIRUP_MILESTONE") {
            object.milestone = x.into();
        }

        if let Ok(options) = airupfx::env::take_var("AIRUP_CONSOLE") {
            for opt in options.split(',') {
                match opt {
                    "quiet" => object.quiet = true,
                    "nocolor" => object.no_color = true,
                    "verbose" => object.verbose = true,
                    _ => {}
                }
            }
        }

        object
    }
}
impl Default for Cmdline {
    fn default() -> Self {
        Self {
            quiet: false,
            no_color: false,
            verbose: false,
            milestone: "default".into(),
        }
    }
}

/// Returns a reference to the unique [`Cmdline`] instance.
pub fn cmdline() -> &'static Cmdline {
    static CMDLINE: OnceLock<Cmdline> = OnceLock::new();

    CMDLINE.get_or_init(|| Cmdline::parse())
}
