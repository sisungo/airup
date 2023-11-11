use airup_sdk::prelude::*;
use anyhow::anyhow;
use clap::Parser;

/// Reload services
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect().await?;
    conn.reload_service(&cmdline.service).await?
        .map_err(|e| anyhow!("failed to reload service `{}`: {}", cmdline.service, e))?;
    Ok(())
}
