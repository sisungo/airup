use airup_sdk::blocking::system::ConnectionExt as _;
use anyhow::anyhow;
use clap::Parser;

/// Stop services
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,

    /// Uncache the service
    #[arg(long)]
    uncache: bool,

    /// Force the service to stop
    #[arg(short, long)]
    force: bool,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect()?;

    let mut stop_service = || {
        if cmdline.force {
            conn.kill_service(&cmdline.service)
        } else {
            conn.stop_service(&cmdline.service)
        }
    };

    if !cmdline.uncache {
        stop_service()?
            .map_err(|e| anyhow!("failed to stop service `{}`: {}", cmdline.service, e))?;
    } else {
        stop_service()?.ok();
    }

    if cmdline.uncache {
        conn.uncache_service(&cmdline.service)?
            .map_err(|e| anyhow!("failed to uncache service `{}`: {}", cmdline.service, e))?;
    }
    Ok(())
}
