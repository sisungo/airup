use airup_sdk::prelude::*;
use clap::Parser;
use console::style;

/// Debug Airup
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    #[arg(short, long)]
    raw: Option<String>,

    #[arg(long)]
    use_logger: Option<String>,

    #[arg(long)]
    print_remote_build_manifest: bool,

    #[arg(long)]
    print_local_build_manifest: bool,

    #[arg(long)]
    internal_crash_handler: bool,

    #[arg(long)]
    trigger_event: Option<String>,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    if cmdline.internal_crash_handler {
        internal_crash_handler();
    }

    if let Some(raw) = cmdline.raw {
        return send_raw(&raw);
    }

    if let Some(logger) = cmdline.use_logger {
        return use_logger(&logger);
    }

    if cmdline.print_remote_build_manifest {
        return print_remote_build_manifest();
    }

    if cmdline.print_local_build_manifest {
        return print_local_build_manifest();
    }

    if let Some(event) = cmdline.trigger_event {
        return trigger_event(&event);
    }

    Ok(())
}

pub fn send_raw(raw: &str) -> anyhow::Result<()> {
    let mut conn = super::connect()?;
    conn.send_raw(raw.as_bytes())?;
    println!("{}", String::from_utf8_lossy(&conn.recv_raw()?));

    Ok(())
}

pub fn use_logger(logger: &str) -> anyhow::Result<()> {
    let mut conn = super::connect()?;
    if logger.is_empty() || logger == "null" {
        conn.use_logger(None)??;
    } else {
        conn.use_logger(Some(logger))??;
    }

    Ok(())
}

pub fn print_remote_build_manifest() -> anyhow::Result<()> {
    let mut conn = super::connect()?;
    let build_manifest = serde_json::to_string_pretty(&conn.build_manifest()??).unwrap();
    println!("{}", build_manifest);

    Ok(())
}

pub fn print_local_build_manifest() -> anyhow::Result<()> {
    let build_manifest = include_str!("../../build_manifest.json");
    println!("{}", build_manifest);

    Ok(())
}

pub fn internal_crash_handler() {
    eprintln!(" {} airupd: crashed", style("ERROR").red());

    if std::process::id() == 1 {
        loop {
            std::hint::spin_loop();
        }
    }

    std::process::exit(255);
}

pub fn trigger_event(event: &str) -> anyhow::Result<()> {
    let mut conn = super::connect()?;
    conn.trigger_event(event)??;

    Ok(())
}
