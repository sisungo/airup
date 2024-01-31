use airup_sdk::{
    blocking::{files::*, fs::DirChain},
    files::{milestone, Milestone},
    system::ConnectionExt as _,
};
use anyhow::anyhow;
use clap::Parser;
use console::style;

/// Disable an unit
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,

    #[arg(short, long)]
    milestone: Option<String>,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let service = cmdline
        .service
        .strip_suffix(".airs")
        .unwrap_or(&cmdline.service);

    let mut conn = super::connect()?;

    let query_system = conn
        .query_system()?
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
        .find(format!("{milestone}.airm"))
        .ok_or_else(|| anyhow!("failed to get milestone `{milestone}`: milestone not found"))?;
    let milestone =
        Milestone::read_from(milestone).map_err(|x| anyhow!("failed to read milestone: {x}"))?;
    let chain = DirChain::new(&milestone.base_dir);

    let path = chain
        .find_or_create("97-auto-generated.list.airf")
        .map_err(|x| anyhow!("failed to open list file: {x}"))?;

    let old = std::fs::read_to_string(&path).unwrap_or_default();
    let mut new = String::with_capacity(old.len());
    let mut disabled = false;

    for x in old.lines() {
        if let Ok(item) = x.parse::<milestone::Item>() {
            match item {
                milestone::Item::Start(x) if x.strip_suffix(".airs").unwrap_or(&x) == service => {
                    disabled = true;
                }
                milestone::Item::Cache(x) if x.strip_suffix(".airs").unwrap_or(&x) == service => {
                    disabled = true;
                }
                _ => {
                    new.push_str(x);
                    new.push('\n');
                }
            };
        }
    }

    std::fs::write(&path, new.as_bytes())?;

    if !disabled {
        eprintln!(
            "{} service {} have not been enabled yet!",
            style("warning:").yellow().bold(),
            service
        );
    }

    Ok(())
}
