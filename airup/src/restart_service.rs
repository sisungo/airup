use airup_sdk::prelude::*;
use clap::Parser;

/// Start services
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = Connection::connect(airup_sdk::socket_path()).await?;
    conn.stop_service(&cmdline.service).await??;
    conn.start_service(&cmdline.service).await??;
    Ok(())
}
