use airup_sdk::prelude::*;
use clap::Parser;

/// Reload `airupd` daemon itself
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    #[arg(long)]
    gc: bool,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect().await?;
    conn.refresh().await??;
    if cmdline.gc {
        conn.gc().await??;
    }
    Ok(())
}
