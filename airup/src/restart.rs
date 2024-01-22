use airup_sdk::system::ConnectionExt as _;
use anyhow::anyhow;
use clap::Parser;

/// Restart services
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,

    /// Restart the service if already started, otherwise start it
    #[arg(short = 'E', long)]
    effective: bool,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect()?;

    let stop = conn.stop_service(&cmdline.service)?;
    if !cmdline.effective {
        stop.map_err(|e| anyhow!("failed to stop service `{}`: {}", cmdline.service, e))?;
    }

    conn.start_service(&cmdline.service)?
        .map_err(|e| anyhow!("failed to start service `{}`: {}", cmdline.service, e))?;
    Ok(())
}
