use airup_sdk::prelude::*;
use clap::Parser;

/// Reload services
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = BlockingConnection::connect(airup_sdk::socket_path())?;
    conn.reload_service(&cmdline.service)??;
    Ok(())
}
