use std::path::{Path, PathBuf};

use airup_sdk::prelude::*;
use anyhow::anyhow;
use clap::Parser;
use console::style;
use rustyline::{error::ReadlineError, history::DefaultHistory};

/// Query system information
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    #[arg(long)]
    sock_repl: bool,

    #[arg(long)]
    auto_reconnect: bool,

    #[arg(short, long)]
    command: Option<String>,
}

pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    if cmdline.sock_repl {
        main_sock_repl(cmdline).await?;
    }
    Ok(())
}

async fn main_sock_repl(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut raw_io = RawIo {
        conn: Connection::connect(airup_sdk::socket_path()).await?,
        rl: rustyline::DefaultEditor::with_config(
            rustyline::Config::builder().auto_add_history(true).build(),
        )?,
    };
    let _ = raw_io.load_history();

    if let Some(cmd) = cmdline.command {
        raw_io.conn.send_raw(cmd.as_bytes()).await?;
        println!(
            "{}",
            String::from_utf8_lossy(&raw_io.conn.recv_raw().await?)
        );
        return Ok(());
    }

    loop {
        if let Err(err) = repl(&mut raw_io).await {
            if cmdline.auto_reconnect {
                eprintln!(
                    "{} disconnected: {}, reconnecting...",
                    style("warning:").yellow(),
                    err
                );
                raw_io.conn.reconnect().await?;
            } else {
                return Err(err);
            }
        }
    }
}

async fn repl(raw_io: &mut RawIo) -> anyhow::Result<()> {
    loop {
        let readline = raw_io.rl.readline("airup> ");
        match readline {
            Ok(line) => {
                raw_io.conn.send_raw(line.as_bytes()).await?;

                println!(
                    "{}",
                    String::from_utf8_lossy(&raw_io.conn.recv_raw().await?)
                );
            }
            Err(err) => match err {
                ReadlineError::Eof => std::process::exit(0),
                ReadlineError::Errno(err) => {
                    eprintln!("{} readline() failed: {}", style("warning:").yellow(), err)
                }
                ReadlineError::Io(err) => {
                    eprintln!("{} readline() failed: {}", style("warning:").yellow(), err)
                }
                _ => {}
            },
        }
    }
}

#[derive(Debug)]
struct RawIo {
    conn: Connection<'static>,
    rl: rustyline::Editor<(), DefaultHistory>,
}
impl RawIo {
    fn load_history(&mut self) -> anyhow::Result<()> {
        self.rl.load_history(&Self::history_path()?)?;
        Ok(())
    }

    fn save_history(&mut self) -> anyhow::Result<()> {
        self.rl.save_history(&Self::history_path()?)?;
        Ok(())
    }

    fn history_path() -> anyhow::Result<PathBuf> {
        match std::env::var("HOME") {
            Ok(x) => Ok(Path::new(&x).join(".airup_rawio_history")),
            Err(_) => Err(anyhow!("environment variable `HOME` is not defined")),
        }
    }
}
impl Drop for RawIo {
    fn drop(&mut self) {
        let _ = self.save_history();
    }
}
