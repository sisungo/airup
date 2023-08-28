use airupfx::{files::Service, sdk::prelude::*};
use clap::Parser;
use std::path::PathBuf;

/// Start services
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,

    #[arg(long)]
    sideload: Option<PathBuf>,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = Connection::connect(airupfx::sdk::socket_path()).await?;
    if let Some(path) = cmdline.sideload {
        conn.sideload_service(&cmdline.service, &Service::read_from(path).await?)
            .await??;
    }
    conn.start_service(&cmdline.service).await??;
    Ok(())
}
