//! # Airup CLI

mod query;
mod raw_io;
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
    RawIo(raw_io::Cmdline),
    Start(start::Cmdline),
    Stop(stop::Cmdline),
    Reload(reload::Cmdline),
    Restart(restart::Cmdline),
    Query(query::Cmdline),
    Poweroff(reboot::Cmdline),
    Reboot(reboot::Cmdline),
    Halt(reboot::Cmdline),
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cmdline = Cmdline::parse();
    let result = match cmdline {
        Cmdline::RawIo(cmdline) => raw_io::main(cmdline).await,
        Cmdline::Start(cmdline) => start::main(cmdline).await,
        Cmdline::Stop(cmdline) => stop::main(cmdline).await,
        Cmdline::Reload(cmdline) => reload::main(cmdline).await,
        Cmdline::Restart(cmdline) => restart::main(cmdline).await,
        Cmdline::Query(cmdline) => query::main(cmdline).await,
        Cmdline::Poweroff(cmdline) => reboot::main(cmdline).await,
        Cmdline::Reboot(cmdline) => reboot::main(cmdline).await,
        Cmdline::Halt(cmdline) => reboot::main(cmdline).await,
    };
    if let Err(e) = result {
        eprintln!("{} {}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}
