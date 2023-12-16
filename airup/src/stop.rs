use airup_sdk::prelude::*;
use anyhow::anyhow;
use clap::Parser;

/// Stop services
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,

    /// Uncache the service
    #[arg(long)]
    uncache: bool,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect().await?;

    if !cmdline.uncache {
        conn.stop_service(&cmdline.service)
            .await?
            .map_err(|e| anyhow!("failed to stop service `{}`: {}", cmdline.service, e))?;
    } else {
        conn.stop_service(&cmdline.service).await?.ok();
    }

    if cmdline.uncache {
        conn.uncache_service(&cmdline.service)
            .await?
            .map_err(|e| anyhow!("failed to uncache service `{}`: {}", cmdline.service, e))?;
    }
    Ok(())
}
