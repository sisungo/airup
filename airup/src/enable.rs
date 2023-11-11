use clap::Parser;

/// Enable an unit
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    unit: String,
}

pub fn main(_: Cmdline) -> anyhow::Result<()> {
    Ok(())
}
