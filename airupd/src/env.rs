//! Inspection and manipulation of `airupd`’s environment.

use std::{borrow::Cow, path::PathBuf, sync::OnceLock};

macro_rules! feed_parser {
    ($flag:ident, $storage:expr, $arg:expr) => {
        if $flag {
            $storage = $arg;
            $flag = false;
            continue;
        }
    };
}

#[derive(Debug, Clone)]
pub struct Cmdline {
    /// Enable verbose console outputs
    pub verbose: bool,

    /// Disable console outputs
    pub quiet: bool,

    /// Disable colorful console outputs
    pub no_color: bool,

    /// Specify bootstrap milestone
    pub milestone: Cow<'static, str>,

    /// Overriding build manifest path
    pub build_manifest: Option<PathBuf>,
}
impl Cmdline {
    /// Parses a new [`Cmdline`] instance from the command-line arguments. This function will automatically detect the
    /// environment to detect the style of the parser.
    pub fn parse() -> Self {
        if cfg!(target_os = "linux") && std::process::id() == 1 {
            Self::parse_as_linux_init()
        } else {
            Self::parse_as_unix_command()
        }
    }

    /// A command-line argument parser that assumes arguments are Linux-init styled.
    fn parse_as_linux_init() -> Self {
        let mut object = Self::default();

        for arg in std::env::args() {
            if arg == "single" {
                object.milestone = "single-user".into();
            }
        }

        if let Ok(x) = airupfx::env::take_var("AIRUP_MILESTONE") {
            object.milestone = x.into();
        }

        if let Ok(options) = airupfx::env::take_var("AIRUP_CONOUT_POLICY") {
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

    /// A command-line argument parser that assumes arguments are passed like common Unix commands.
    fn parse_as_unix_command() -> Self {
        let mut object = Self::default();

        let mut parsing_milestone = false;
        let mut parsing_build_manifest = false;
        for arg in std::env::args() {
            feed_parser!(parsing_milestone, object.milestone, arg.into());
            feed_parser!(
                parsing_build_manifest,
                object.build_manifest,
                Some(arg.into())
            );

            match &arg[..] {
                "-m" | "--milestone" => parsing_milestone = true,
                "--build-manifest" => parsing_build_manifest = true,
                "--verbose" => object.verbose = true,
                "-q" | "--quiet" => object.quiet = true,
                "--no-color" => object.no_color = true,
                "-h" | "--help" => Self::print_help(),
                "-V" | "--version" => Self::print_version(),
                _ => (),
            }
        }

        object
    }

    fn print_help() -> ! {
        println!("Usage: airupd [OPTIONS]");
        println!();
        println!("Options:");
        println!("    -h, --help                        Print help");
        println!("    -V, --version                     Print version");
        println!("        --build-manifest <PATH>       Override builtin build manifest");
        println!("    -m, --milestone <NAME>            Specify bootstrap milestone");
        println!("    -q, --quiet                       Disable console outputs");
        println!("        --verbose                     Enable verbose console outputs");
        println!("        --no-color                    Disable colorful console outputs");
        std::process::exit(0);
    }

    fn print_version() -> ! {
        println!("airupd v{}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }
}
impl Default for Cmdline {
    fn default() -> Self {
        Self {
            quiet: false,
            no_color: false,
            verbose: false,
            milestone: "default".into(),
            build_manifest: None,
        }
    }
}

/// Returns a reference to the unique [`Cmdline`] instance.
pub fn cmdline() -> &'static Cmdline {
    static CMDLINE: OnceLock<Cmdline> = OnceLock::new();

    CMDLINE.get_or_init(Cmdline::parse)
}
