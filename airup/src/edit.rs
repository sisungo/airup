use clap::Parser;

/// Edit Airup files
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    file: String,
}

pub fn main(_: Cmdline) -> anyhow::Result<()> {
    Ok(())
}
