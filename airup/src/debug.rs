use airup_sdk::prelude::*;
use clap::Parser;

/// Debug Airup
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    #[arg(short, long)]
    command: Option<String>,
}

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = BlockingConnection::connect(airup_sdk::socket_path())?;
    if let Some(cmd) = cmdline.command {
        conn.send_raw(cmd.as_bytes())?;
        println!("{}", String::from_utf8_lossy(&conn.recv_raw()?));
        return Ok(());
    }
    Ok(())
}
