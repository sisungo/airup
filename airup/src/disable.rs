use airup_sdk::{
    files::{milestone, Milestone},
    fs::DirChain,
    system::ConnectionExt,
};
use anyhow::anyhow;
use clap::Parser;
use console::style;
use tokio::io::{AsyncBufReadExt, BufReader};

/// Disable an unit
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,

    #[arg(short, long)]
    milestone: Option<String>,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let service = cmdline
        .service
        .strip_suffix(".airs")
        .unwrap_or(&cmdline.service);

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
        .find(&format!("{milestone}.airm"))
        .await
        .ok_or_else(|| anyhow!("failed to get milestone `{milestone}`: milestone not found"))?;
    let milestone = Milestone::read_from(milestone)
        .await
        .map_err(|x| anyhow!("failed to read milestone: {x}"))?;

    let path = milestone
        .base_chain
        .find_or_create("97-auto-generated.list.airf")
        .await
        .map_err(|x| anyhow!("failed to open list file: {x}"))?;
    let file = tokio::fs::File::options()
        .create(true)
        .write(true)
        .read(true)
        .open(&path)
        .await
        .map_err(|x| anyhow!("failed to open list file: {x}"))?;

    let mut new = String::with_capacity(file.metadata().await?.len() as usize);
    let mut lines = BufReader::new(file).lines();
    let mut disabled = false;
    while let Ok(Some(x)) = lines.next_line().await {
        if let Ok(item) = x.parse::<milestone::Item>() {
            match item {
                milestone::Item::Start(x) if x.strip_suffix(".airs").unwrap_or(&x) == service => {
                    disabled = true;
                }
                milestone::Item::Cache(x) if x.strip_suffix(".airs").unwrap_or(&x) == service => {
                    disabled = true;
                }
                _ => {
                    new.push_str(&x);
                    new.push('\n');
                }
            };
        }
    }

    tokio::fs::write(&path, new.as_bytes()).await?;

    if !disabled {
        eprintln!(
            "{} service {} have not been enabled yet!",
            style("warning:").yellow().bold(),
            service
        );
    }

    Ok(())
}
