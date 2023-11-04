use airup_sdk::prelude::*;
use clap::Parser;

/// Disable an unit
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    unit: String,
}

pub fn main(_: Cmdline) -> anyhow::Result<()> {
    let mut _conn = BlockingConnection::connect(airup_sdk::socket_path())?;
    Ok(())
}
