use airup_sdk::prelude::*;
use clap::Parser;

/// Edit Airup files
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    file: String,
}

pub fn main(_: Cmdline) -> anyhow::Result<()> {
    let mut _conn = BlockingConnection::connect(airup_sdk::socket_path())?;
    Ok(())
}
