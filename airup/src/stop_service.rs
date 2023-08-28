use airupfx::sdk::prelude::*;
use clap::Parser;

/// Stop services
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = Connection::connect(airupfx::sdk::socket_path()).await?;
    conn.stop_service(&cmdline.service).await??;
    Ok(())
}
