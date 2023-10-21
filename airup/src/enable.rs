use airup_sdk::prelude::*;
use clap::Parser;

/// Enable an unit
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    unit: String,
}

pub async fn main(_: Cmdline) -> anyhow::Result<()> {
    let mut _conn = Connection::connect(airup_sdk::socket_path()).await?;
    Ok(())
}
