use airup_sdk::blocking::system::ConnectionExt as _;
use anyhow::anyhow;
use clap::Parser;

/// Reload services
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect()?;
    conn.reload_service(&cmdline.service)?
        .map_err(|e| anyhow!("failed to reload service `{}`: {}", cmdline.service, e))?;
    Ok(())
}
