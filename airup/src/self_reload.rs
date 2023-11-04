use airup_sdk::prelude::*;
use clap::Parser;

/// Reload `airupd` daemon itself
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {}

pub fn main(_: Cmdline) -> anyhow::Result<()> {
    let mut conn = BlockingConnection::connect(airup_sdk::socket_path())?;
    conn.refresh()??;
    conn.gc()??;
    Ok(())
}
