use airup_sdk::blocking::{info::ConnectionExt as _, system::ConnectionExt as _};
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

pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect()?;

    if let Some(cmd) = cmdline.raw {
        conn.send_raw(cmd.as_bytes())?;
        println!("{}", String::from_utf8_lossy(&conn.recv_raw()?));
        return Ok(());
    }

    if let Some(logger) = cmdline.use_logger {
        if logger.is_empty() {
            conn.use_logger(None)??;
        } else {
            conn.use_logger(Some(&logger))??;
        }
    }

    if cmdline.print_build_manifest {
        println!(
            "{}",
            serde_json::to_string_pretty(&conn.build_manifest()??).unwrap()
        );
        return Ok(());
    }

    Ok(())
}
