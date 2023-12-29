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

    /// Force the service to stop
    #[arg(short, long)]
    force: bool,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect().await?;

    let stop_service = async {
        if cmdline.force {
            conn.kill_service(&cmdline.service).await
        } else {
            conn.stop_service(&cmdline.service).await
        }
    };

    if !cmdline.uncache {
        stop_service
            .await?
            .map_err(|e| anyhow!("failed to stop service `{}`: {}", cmdline.service, e))?;
    } else {
        stop_service.await?.ok();
    }

    if cmdline.uncache {
        conn.uncache_service(&cmdline.service)
            .await?
            .map_err(|e| anyhow!("failed to uncache service `{}`: {}", cmdline.service, e))?;
    }
    Ok(())
}
