//! Inspection and manipulation of `airupd`’s environment.

use std::borrow::Cow;
use std::sync::OnceLock;

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

    /// Start Airupd in user mode (experimental)
    pub user: bool,
}
impl Cmdline {
    /// Parses a new [`Cmdline`] instance from the command-line arguments. This function will automatically detect the
    /// environment to detect the style of the parser.
    pub fn parse() -> Self {
        if cfg!(target_os = "linux") || *airupfx::process::ID == 1 {
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

    /// A command-line argument parser that assumes arguments are passed like common Unix commands.
    fn parse_as_unix_command() -> Self {
        let mut object = Self::default();

        let mut parsing_milestone = false;
        for arg in std::env::args() {
            if parsing_milestone {
                object.milestone = arg.into();
                parsing_milestone = false;
                continue;
            }

            if arg == "-m" || arg == "--milestone" {
                parsing_milestone = true;
            } else if arg == "-u" || arg == "--user" {
                object.user = true;
            } else if arg == "--verbose" {
                object.verbose = true;
            } else if arg == "-q" || arg == "--quiet" {
                object.quiet = true;
            } else if arg == "--no-color" {
                object.no_color = true;
            } else if arg == "-h" || arg == "--help" {
                Self::print_help();
            } else if arg == "-V" || arg == "--version" {
                Self::print_version();
            }

            let mut parsing_milestone2 = None;
            if let Some(x) = arg.strip_prefix("-m") {
                parsing_milestone2 = Some(x);
            } else if let Some(x) = arg.strip_prefix("--milestone=") {
                parsing_milestone2 = Some(x);
            }

            if let Some(x) = parsing_milestone2 {
                object.milestone = String::from(x).into();
                continue;
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
        println!("    -m, --milestone <NAME>            Specify bootstrap milestone");
        println!("    -u, --user                        Run `airupd` in user mode (experimental)");
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
            user: false,
        }
    }
}

/// Returns a reference to the unique [`Cmdline`] instance.
pub fn cmdline() -> &'static Cmdline {
    static CMDLINE: OnceLock<Cmdline> = OnceLock::new();

    CMDLINE.get_or_init(Cmdline::parse)
}
