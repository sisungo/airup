use airup_sdk::{files::Milestone, fs::DirChain, system::ConnectionExt};
use anyhow::anyhow;
use clap::Parser;
use console::style;
use tokio::io::AsyncWriteExt;

/// Enable an unit
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    unit: String,

    #[arg(short, long)]
    cache: bool,

    #[arg(short, long)]
    milestone: Option<String>,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect().await?;
    let query_system = conn
        .query_system()
        .await?
        .map_err(|x| anyhow!("failed to query system information: {x}"))?;
    let current_milestone = query_system
        .milestones
        .last()
        .map(|x| &x.name[..])
        .unwrap_or_else(|| {
            eprintln!(
                "{} failed to get current milestone, writing to `default` milestone!",
                style("warning:").yellow().bold()
            );
            "default"
        });
    let milestones = DirChain::new(&airup_sdk::build::manifest().milestone_dir);
    let milestone = cmdline
        .milestone
        .unwrap_or_else(|| current_milestone.into());
    let milestone = milestones
        .find(&milestone)
        .await
        .ok_or_else(|| anyhow!("failed to get milestone {milestone}: milestone not found"))?;
    let milestone = Milestone::read_from(milestone)
        .await
        .map_err(|x| anyhow!("failed to read milestone: {x}"))?;
    let file = milestone
        .base_chain
        .find_or_create("97-auto-generated.list.airf")
        .await
        .map_err(|x| anyhow!("failed to open list file: {x}"))?;
    let mut file = tokio::fs::File::options()
        .write(true)
        .create(true)
        .append(true)
        .open(file)
        .await
        .map_err(|x| anyhow!("failed to open list file: {x}"))?;
    if cmdline.cache {
        file.write_all(format!("\ncache {}\n", cmdline.unit).as_bytes()).await?;
    } else {
        file.write_all(format!("\nstart {}\n", cmdline.unit).as_bytes()).await?;
    }
    Ok(())
}
