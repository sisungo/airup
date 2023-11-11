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

pub fn connect() -> anyhow::Result<airup_sdk::BlockingConnection> {
    Ok(
        airup_sdk::BlockingConnection::connect(airup_sdk::socket_path())
            .map_err(|e| anyhow!("unable to communicate with airup daemon: {}", e))?,
    )
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

fn main() {
    let cmdline = Cmdline::parse();
    let result = match cmdline {
        Cmdline::Start(cmdline) => start::main(cmdline),
        Cmdline::Stop(cmdline) => stop::main(cmdline),
        Cmdline::Reload(cmdline) => reload::main(cmdline),
        Cmdline::Restart(cmdline) => restart::main(cmdline),
        Cmdline::Query(cmdline) => query::main(cmdline),
        Cmdline::Reboot(cmdline) => reboot::main(cmdline),
        Cmdline::SelfReload(cmdline) => self_reload::main(cmdline),
        Cmdline::Edit(cmdline) => edit::main(cmdline),
        Cmdline::Enable(cmdline) => enable::main(cmdline),
        Cmdline::Disable(cmdline) => disable::main(cmdline),
        Cmdline::Debug(cmdline) => debug::main(cmdline),
    };

    if let Err(e) = result {
        eprintln!("airup: {} {}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}
