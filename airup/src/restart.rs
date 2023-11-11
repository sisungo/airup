use airup_sdk::prelude::*;
use anyhow::anyhow;
use clap::Parser;

/// Restart services
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect().await?;
    conn.stop_service(&cmdline.service).await?
        .map_err(|e| anyhow!("failed to stop service `{}`: {}", cmdline.service, e))?;
    conn.start_service(&cmdline.service).await?
        .map_err(|e| anyhow!("failed to start service `{}`: {}", cmdline.service, e))?;
    Ok(())
}
