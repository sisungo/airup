use airup_sdk::{files::Service, prelude::*};
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

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect()?;
    if let Some(path) = cmdline.sideload {
        conn.sideload_service(&cmdline.service, &Service::read_from_blocking(path)?)??;
    }
    conn.start_service(&cmdline.service)??;
    Ok(())
}
