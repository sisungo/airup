use airup_sdk::{files::Service, prelude::*};
use anyhow::anyhow;
use clap::Parser;
use std::path::PathBuf;

/// Start services
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,

    /// Cache the service only
    #[arg(long)]
    cache: bool,

    /// Sideload a service
    #[arg(long)]
    sideload: Option<PathBuf>,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect().await?;

    let queried = conn.query_service(&cmdline.service).await?;
    let would_interrupt = queried
        .map(|x| {
            x.task_name.as_deref() == Some("CleanupService")
                && x.task_class.as_deref() == Some("StartService")
        })
        .unwrap_or_default();

    if would_interrupt {
        conn.interrupt_service_task(&cmdline.service).await?.ok();
    }

    if let Some(path) = &cmdline.sideload {
        let service = Service::read_merge(&[path])
            .await
            .map_err(|e| anyhow!("failed to read service at `{}`: {}", path.display(), e))?;
        conn.sideload_service(&cmdline.service, &service).await??;
    }

    if !cmdline.cache {
        conn.start_service(&cmdline.service)
            .await?
            .map_err(|e| anyhow!("failed to start service `{}`: {}", cmdline.service, e))?;
    } else if cmdline.sideload.is_none() {
        conn.cache_service(&cmdline.service)
            .await?
            .map_err(|e| anyhow!("failed to cache service `{}`: {}", cmdline.service, e))?;
    }
    Ok(())
}
