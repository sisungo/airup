use airup_sdk::system::ConnectionExt as _;
use clap::Parser;

/// Reload the `airupd` daemon itself
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    /// Run garbage collector
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
