use airup_sdk::{
    blocking::{files::*, fs::DirChain, system::ConnectionExt as _},
    files::{milestone, Milestone},
};
use anyhow::anyhow;
use clap::Parser;
use console::style;
use std::io::Write;

/// Enable an unit
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    service: String,

    #[arg(short, long)]
    force: bool,

    #[arg(short, long)]
    cache: bool,

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
        .find(&format!("{milestone}.airm"))
        .ok_or_else(|| anyhow!("failed to get milestone `{milestone}`: milestone not found"))?;
    let milestone =
        Milestone::read_from(milestone).map_err(|x| anyhow!("failed to read milestone: {x}"))?;
    let chain = DirChain::new(&milestone.base_dir);

    for item in milestone.items() {
        match item {
            milestone::Item::Start(x) if x.strip_suffix(".airs").unwrap_or(&x) == service => {
                eprintln!(
                    "{} service {} have already been enabled",
                    style("warning:").yellow().bold(),
                    service
                );
                std::process::exit(0);
            }
            milestone::Item::Cache(x)
                if x.strip_suffix(".airs").unwrap_or(&x) == service && cmdline.cache =>
            {
                eprintln!(
                    "{} service {} have already been enabled",
                    style("warning:").yellow().bold(),
                    service
                );
                std::process::exit(0);
            }
            _ => (),
        }
    }

    if !cmdline.force {
        conn.query_service(service)?
            .map_err(|x| anyhow!("failed to enable service `{}`: {}", service, x))?;
    }

    let file = chain
        .find_or_create("97-auto-generated.list.airf")
        .map_err(|x| anyhow!("failed to open list file: {x}"))?;
    let mut file = std::fs::File::options()
        .write(true)
        .create(true)
        .append(true)
        .open(file)
        .map_err(|x| anyhow!("failed to open list file: {x}"))?;

    if cmdline.cache {
        file.write_all(format!("\ncache {}\n", service).as_bytes())
            .map_err(|x| anyhow!("failed to write to list file: {x}"))?;
    } else {
        file.write_all(format!("\nstart {}\n", service).as_bytes())
            .map_err(|x| anyhow!("failed to write to list file: {x}"))?;
    }

    Ok(())
}
