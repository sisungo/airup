use airup_sdk::prelude::*;
use anyhow::anyhow;
use clap::Parser;

/// Restart services
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,

    /// Restart the service if already started, otherwise start it
    #[arg(short = 'E', long)]
    effective: bool,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect().await?;

    let stop = conn.stop_service(&cmdline.service).await?;
    if !cmdline.effective {
        stop.map_err(|e| anyhow!("failed to stop service `{}`: {}", cmdline.service, e))?;
    }

    conn.start_service(&cmdline.service)
        .await?
        .map_err(|e| anyhow!("failed to start service `{}`: {}", cmdline.service, e))?;
    Ok(())
}
