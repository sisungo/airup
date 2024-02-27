use airup_sdk::{blocking::files::*, files::Service, system::ConnectionExt as _};
use anyhow::anyhow;
use clap::Parser;
use std::path::PathBuf;

/// Start a service
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

    #[arg(long = "override", requires = "sideload")]
    ovrd: bool,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect()?;

    if let Some(path) = &cmdline.sideload {
        let service = Service::read_merge(vec![path.clone()])
            .map_err(|e| anyhow!("failed to read service at `{}`: {}", path.display(), e))?;
        conn.sideload_service(&cmdline.service, &service, cmdline.ovrd)??;
    }

    if !cmdline.cache {
        conn.start_service(&cmdline.service)?
            .map_err(|e| anyhow!("failed to start service `{}`: {}", cmdline.service, e))?;
    } else if cmdline.sideload.is_none() {
        conn.cache_service(&cmdline.service)?
            .map_err(|e| anyhow!("failed to cache service `{}`: {}", cmdline.service, e))?;
    }
    Ok(())
}
