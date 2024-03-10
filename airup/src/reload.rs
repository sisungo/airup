use airup_sdk::system::ConnectionExt as _;
use anyhow::anyhow;
use clap::Parser;

/// Notify a service to reload its status
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect()?;

    if let "airupd" | "airupd.airs" = &cmdline.service[..] {
        if let Some("& airup self-reload") = conn
            .query_service(&cmdline.service)??
            .definition
            .exec
            .reload
            .as_deref()
        {
            conn.refresh()??;
        }
    }

    conn.reload_service(&cmdline.service)?
        .map_err(|e| anyhow!("failed to reload service `{}`: {}", cmdline.service, e))?;

    Ok(())
}
