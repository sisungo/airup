use std::collections::HashSet;

use airup_sdk::prelude::*;
use clap::Parser;

/// Debug options of Airup
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    #[arg(long)]
    use_logger: Option<String>,

    #[arg(long)]
    print_remote_build_manifest: bool,

    #[arg(long)]
    print_local_build_manifest: bool,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    if let Some(logger) = cmdline.use_logger {
        return use_logger(&logger);
    }

    if cmdline.print_remote_build_manifest {
        return print_remote_build_manifest();
    }

    if cmdline.print_local_build_manifest {
        return print_local_build_manifest();
    }

    Ok(())
}

pub fn use_logger(logger: &str) -> anyhow::Result<()> {
    let mut conn = super::connect()?;
    let mut logger_methods = HashSet::with_capacity(2);
    logger_methods.insert("logger.append".into());
    logger_methods.insert("logger.tail".into());
    conn.load_extension("logger", &[logger.into()], logger_methods)??;

    Ok(())
}

pub fn print_remote_build_manifest() -> anyhow::Result<()> {
    let mut conn = super::connect()?;
    let build_manifest = serde_json::to_string_pretty(&conn.build_manifest()??).unwrap();
    println!("{}", build_manifest);

    Ok(())
}

pub fn print_local_build_manifest() -> anyhow::Result<()> {
    let build_manifest = include_str!("../../build_manifest.json");
    println!("{}", build_manifest);

    Ok(())
}
