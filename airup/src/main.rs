//! # Airup CLI

mod debug;
mod edit;
mod query;
mod reboot;
mod reload;
mod restart;
mod start;
mod stop;

use clap::Parser;
use console::style;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub enum Cmdline {
    Start(start::Cmdline),
    Stop(stop::Cmdline),
    Reload(reload::Cmdline),
    Restart(restart::Cmdline),
    Query(query::Cmdline),
    Poweroff(reboot::Cmdline),
    Reboot(reboot::Cmdline),
    Halt(reboot::Cmdline),
    Edit(edit::Cmdline),
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
        Cmdline::Poweroff(cmdline) => reboot::main(cmdline).await,
        Cmdline::Reboot(cmdline) => reboot::main(cmdline).await,
        Cmdline::Halt(cmdline) => reboot::main(cmdline).await,
        Cmdline::Edit(cmdline) => edit::main(cmdline).await,
        Cmdline::Debug(cmdline) => debug::main(cmdline).await,
    };
    if let Err(e) = result {
        eprintln!("{} {}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}
