//! Command-line utility for accessing Airup facilities.

mod daemon;
mod debug;
mod disable;
mod edit;
mod enable;
mod query;
mod reboot;
mod reload;
mod restart;
mod self_reload;
mod start;
mod stop;
mod trigger_event;
mod util;

use anyhow::anyhow;
use clap::Parser;
use console::style;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
enum Subcommand {
    Start(start::Cmdline),
    Stop(stop::Cmdline),
    Reload(reload::Cmdline),
    Restart(restart::Cmdline),
    Query(query::Cmdline),
    SelfReload(self_reload::Cmdline),
    Reboot(reboot::Cmdline),
    Edit(edit::Cmdline),
    Enable(enable::Cmdline),
    Disable(disable::Cmdline),
    TriggerEvent(trigger_event::Cmdline),
    Daemon(daemon::Cmdline),
    Debug(debug::Cmdline),
}
impl Subcommand {
    fn execute(self) -> anyhow::Result<()> {
        match self {
            Self::Start(cmdline) => start::main(cmdline),
            Self::Stop(cmdline) => stop::main(cmdline),
            Self::Reload(cmdline) => reload::main(cmdline),
            Self::Restart(cmdline) => restart::main(cmdline),
            Self::Query(cmdline) => query::main(cmdline),
            Self::Reboot(cmdline) => reboot::main(cmdline),
            Self::SelfReload(cmdline) => self_reload::main(cmdline),
            Self::Edit(cmdline) => edit::main(cmdline),
            Self::Enable(cmdline) => enable::main(cmdline),
            Self::Disable(cmdline) => disable::main(cmdline),
            Self::TriggerEvent(cmdline) => trigger_event::main(cmdline),
            Self::Daemon(cmdline) => daemon::main(cmdline),
            Self::Debug(cmdline) => debug::main(cmdline),
        }
    }
}

#[derive(Parser)]
struct Cmdline {
    #[command(subcommand)]
    subcommand: Subcommand,

    /// Override default build manifest
    #[arg(long)]
    build_manifest: Option<PathBuf>,
}
impl Cmdline {
    fn execute(self) -> anyhow::Result<()> {
        set_build_manifest_at(self.build_manifest.as_deref())?;
        self.subcommand.execute()?;

        Ok(())
    }
}

fn main() {
    let cmdline = Cmdline::parse();

    if let Err(e) = cmdline.execute() {
        eprintln!("{} {}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}

pub fn connect() -> anyhow::Result<airup_sdk::blocking::Connection> {
    airup_sdk::blocking::Connection::connect(airup_sdk::socket_path())
        .map_err(|e| anyhow!("cannot connect to airup daemon: {}", e))
}

fn set_build_manifest_at(path: Option<&Path>) -> anyhow::Result<()> {
    if let Some(path) = path {
        airup_sdk::build::set_manifest(
            serde_json::from_slice(
                &std::fs::read(path)
                    .map_err(|err| anyhow!("failed to read overridden build manifest: {err}"))?,
            )
            .map_err(|err| anyhow!("failed to parse overridden build manifest: {err}"))?,
        );
    }

    Ok(())
}
