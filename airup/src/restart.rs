use airup_sdk::prelude::*;
use clap::Parser;

/// Restart services
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect()?;
    conn.stop_service(&cmdline.service)??;
    conn.start_service(&cmdline.service)??;
    Ok(())
}
