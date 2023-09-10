use airup_sdk::prelude::*;
use airupfx::files::Service;
use clap::Parser;

/// Start services
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = Connection::connect(airup_sdk::socket_path()).await?;
    if cmdline.service.strip_suffix(Service::SUFFIX).unwrap_or(&cmdline.service) == "airupd" {
        conn.refresh().await??;
    } else {
        conn.reload_service(&cmdline.service).await??;
    }
    Ok(())
}
