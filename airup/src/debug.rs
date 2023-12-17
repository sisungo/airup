use airup_sdk::{info::ConnectionExt as _, system::ConnectionExt as _};
use clap::Parser;

/// Debug Airup
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    #[arg(short, long)]
    raw: Option<String>,

    #[arg(long)]
    use_logger: Option<String>,

    #[arg(long)]
    print_build_manifest: bool,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect().await?;

    if let Some(cmd) = cmdline.raw {
        conn.send_raw(cmd.as_bytes()).await?;
        println!("{}", String::from_utf8_lossy(&conn.recv_raw().await?));
        return Ok(());
    }

    if let Some(logger) = cmdline.use_logger {
        if logger.is_empty() {
            conn.use_logger(None).await??;
        } else {
            conn.use_logger(Some(&logger)).await??;
        }
    }

    if cmdline.print_build_manifest {
        println!(
            "{}",
            serde_json::to_string_pretty(&conn.build_manifest().await??).unwrap()
        );
        return Ok(());
    }

    Ok(())
}
