use airup_sdk::prelude::*;
use clap::Parser;

/// Debug Airup
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    #[arg(short, long)]
    command: Option<String>,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = Connection::connect(airup_sdk::socket_path()).await?;
    if let Some(cmd) = cmdline.command {
        conn.send_raw(cmd.as_bytes()).await?;
        println!(
            "{}",
            String::from_utf8_lossy(&conn.recv_raw().await?)
        );
        return Ok(());
    }
    Ok(())
}