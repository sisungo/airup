//! Inspection and manipulation of `airupd`’s environment.

use std::borrow::Cow;
use std::sync::OnceLock;

static CMDLINE: OnceLock<Cmdline> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct Cmdline {
    /// Disable Airupd console outputs
    pub quiet: bool,

    /// Disable colors for Airupd console outputs
    pub no_color: bool,

    /// Specify Airup milestone
    pub milestone: Cow<'static, str>,
}
impl Cmdline {
    /// Parses command-line arguments for use of [cmdline].
    pub fn init() {
        CMDLINE.set(Self::parse()).unwrap();
    }

    /// Parses a new [Cmdline] instance from the command-line arguments.
    pub fn parse() -> Self {
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
            milestone: "default".into(),
        }
    }
}

/// Returns a reference to the unique [Cmdline].
pub fn cmdline() -> &'static Cmdline {
    CMDLINE.get().unwrap()
}
