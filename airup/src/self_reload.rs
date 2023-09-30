use airup_sdk::prelude::*;
use clap::Parser;

/// Reload `airupd` daemon itself
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {}

pub async fn main(_: Cmdline) -> anyhow::Result<()> {
    let mut conn = Connection::connect(airup_sdk::socket_path()).await?;
    conn.refresh().await??;
    conn.gc().await??;
    Ok(())
}
