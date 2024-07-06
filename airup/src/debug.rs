use airup_sdk::{debug::ConnectionExt, prelude::*};
use anyhow::anyhow;
use clap::{Parser, ValueEnum};

/// Debug options of Airup
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    /// Print build manifest acquired from Airup daemon
    #[arg(long)]
    print: Option<Printable>,

    /// Reduce RPC if possible
    #[arg(long)]
    reduce_rpc: bool,

    /// Unregister an Airup extension
    #[arg(long)]
    unregister_extension: Option<String>,

    /// Dump Airup's internal debug information
    #[arg(long)]
    dump: bool,

    /// Sets the server's instance name
    #[arg(long)]
    set_instance_name: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Printable {
    BuildManifest,
}
impl Printable {
    fn print(self, reduce_rpc: bool) -> anyhow::Result<()> {
        match self {
            Self::BuildManifest => print_build_manifest(reduce_rpc),
        }
    }
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    if let Some(printable) = cmdline.print {
        printable.print(cmdline.reduce_rpc)
    } else if let Some(name) = cmdline.unregister_extension {
        unregister_extension(&name)
    } else if cmdline.dump {
        dump()
    } else if let Some(name) = cmdline.set_instance_name {
        set_instance_name(&name)
    } else {
        Err(anyhow!("no action specified"))
    }
}

pub fn dump() -> anyhow::Result<()> {
    let mut conn = super::connect()?;
    let reply = conn.dump()??;

    println!("{reply}");

    Ok(())
}

pub fn set_instance_name(name: &str) -> anyhow::Result<()> {
    let mut conn = super::connect()?;
    conn.set_instance_name(name)??;
    Ok(())
}

pub fn unregister_extension(name: &str) -> anyhow::Result<()> {
    let mut conn = super::connect()?;
    conn.unregister_extension(name)??;

    Ok(())
}

pub fn print_build_manifest(reduce_rpc: bool) -> anyhow::Result<()> {
    let build_manifest = if reduce_rpc {
        serde_json::to_string_pretty(airup_sdk::build::manifest())
            .expect("failed to serialize `airup_sdk::build::manifest()` into JSON")
    } else {
        serde_json::to_string_pretty(&super::connect()?.build_manifest()??)
            .expect("failed to serialize server side `BuildManifest` into JSON")
    };
    println!("{}", build_manifest);

    Ok(())
}
