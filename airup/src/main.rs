//! # Airup CLI

mod query_service;
mod raw_io;
mod reboot;
mod reload_service;
mod restart_service;
mod start_service;
mod stop_service;

use clap::Parser;
use console::style;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub enum Cmdline {
    RawIo(raw_io::Cmdline),
    StartService(start_service::Cmdline),
    StopService(stop_service::Cmdline),
    ReloadService(reload_service::Cmdline),
    RestartService(restart_service::Cmdline),
    QueryService(query_service::Cmdline),
    Shutdown(reboot::Cmdline),
    Reboot(reboot::Cmdline),
    Halt(reboot::Cmdline),
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let cmdline = Cmdline::parse();
    let result = match cmdline {
        Cmdline::RawIo(cmdline) => raw_io::main(cmdline).await,
        Cmdline::StartService(cmdline) => start_service::main(cmdline).await,
        Cmdline::StopService(cmdline) => stop_service::main(cmdline).await,
        Cmdline::ReloadService(cmdline) => reload_service::main(cmdline).await,
        Cmdline::RestartService(cmdline) => restart_service::main(cmdline).await,
        Cmdline::QueryService(cmdline) => query_service::main(cmdline).await,
        Cmdline::Shutdown(cmdline) => reboot::main(cmdline).await,
        Cmdline::Reboot(cmdline) => reboot::main(cmdline).await,
        Cmdline::Halt(cmdline) => reboot::main(cmdline).await,
    };
    if let Err(e) = result {
        eprintln!("{} {}", style("error:").red().bold(), e);
        std::process::exit(1);
    }
}
