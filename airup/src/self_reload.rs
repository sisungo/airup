use airup_sdk::prelude::*;
use clap::Parser;

/// Reload `airupd` daemon itself
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    #[arg(long)]
    gc: bool,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect()?;
    conn.refresh()??;
    if cmdline.gc {
        conn.gc()??;
    }
    Ok(())
}
