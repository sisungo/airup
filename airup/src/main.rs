//! # Airup CLI

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

use anyhow::anyhow;
use clap::Parser;
use console::style;

pub async fn connect() -> anyhow::Result<airup_sdk::Connection> {
    airup_sdk::Connection::connect(airup_sdk::socket_path())
        .await
        .map_err(|e| anyhow!("cannot connect to airup daemon: {}", e))
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub enum Cmdline {
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
    Debug(debug::Cmdline),
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cmdline = Cmdline::parse();
    let result = match cmdline {
        Cmdline::Start(cmdline) => start::main(cmdline).await,
        Cmdline::Stop(cmdline) => stop::main(cmdline).await,
        Cmdline::Reload(cmdline) => reload::main(cmdline).await,
        Cmdline::Restart(cmdline) => restart::main(cmdline).await,
        Cmdline::Query(cmdline) => query::main(cmdline).await,
        Cmdline::Reboot(cmdline) => reboot::main(cmdline).await,
        Cmdline::SelfReload(cmdline) => self_reload::main(cmdline).await,
        Cmdline::Edit(cmdline) => edit::main(cmdline).await,
        Cmdline::Enable(cmdline) => enable::main(cmdline).await,
        Cmdline::Disable(cmdline) => disable::main(cmdline).await,
        Cmdline::Debug(cmdline) => debug::main(cmdline).await,
    };

    if let Err(e) = result {
        eprintln!("{} {}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}
