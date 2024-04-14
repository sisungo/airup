use airup_sdk::prelude::*;
use clap::Parser;

/// Debug options of Airup
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    #[arg(long)]
    print_remote_build_manifest: bool,

    #[arg(long)]
    print_local_build_manifest: bool,

    #[arg(long)]
    unload_extension: Option<String>,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    if cmdline.print_remote_build_manifest {
        return print_remote_build_manifest();
    }

    if cmdline.print_local_build_manifest {
        return print_local_build_manifest();
    }

    if let Some(name) = cmdline.unload_extension {
        return unload_extension(&name);
    }

    Ok(())
}

pub fn unload_extension(name: &str) -> anyhow::Result<()> {
    let mut conn = super::connect()?;
    conn.unload_extension(name)??;

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
