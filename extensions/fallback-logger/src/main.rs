//! A simple logger.
//!
//! This has some limitations and has bad performance. Being designed as an "fallback choice", the implementation aims to be
//! small.

use airup_sdk::{blocking::fs::DirChain, system::LogRecord, Error};
use airupfx::extensions::*;
use rev_lines::RevLines;
use std::io::Write;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    Server::new("logger")
        .await?
        .mount("append", append)
        .mount("tail", tail)
        .run()
        .await
}

#[airupfx::macros::api]
async fn append(subject: String, module: String, msg: Vec<u8>) -> Result<(), Error> {
    let mut appender = open_subject_append(&subject).map_err(|x| Error::Io {
        message: x.to_string(),
    })?;
    let timestamp = airupfx::time::timestamp_ms();

    for m in msg.split(|x| b"\r\n\0".contains(x)) {
        let record = LogRecord {
            timestamp,
            module: module.to_owned(),
            message: String::from_utf8_lossy(m).into_owned(),
        };
        writeln!(
            appender,
            "{}",
            serde_json::to_string(&record).unwrap().as_str()
        )
        .map_err(|x| airup_sdk::Error::Io {
            message: x.to_string(),
        })?;
    }

    Ok(())
}

#[airupfx::macros::api]
async fn tail(subject: String, n: usize) -> Result<Vec<LogRecord>, Error> {
    let reader = open_subject_read(&subject).map_err(|x| Error::Io {
        message: x.to_string(),
    })?;

    let mut result = Vec::with_capacity(n);
    for line in RevLines::new(reader).take(n) {
        result.push(
            serde_json::from_str(&line.map_err(|x| Error::Io {
                message: x.to_string(),
            })?)
            .map_err(|x| Error::Io {
                message: x.to_string(),
            })?,
        );
    }
    result.reverse();
    Ok(result)
}

fn dir_chain_logs() -> DirChain<'static> {
    DirChain::new(&airup_sdk::build::manifest().log_dir)
}

fn open_subject_append(subject: &str) -> std::io::Result<std::fs::File> {
    let path = dir_chain_logs().find_or_create(format!("{subject}.fallback_logger.json"))?;

    std::fs::File::options()
        .append(true)
        .create(true)
        .open(path)
}

fn open_subject_read(subject: &str) -> std::io::Result<std::fs::File> {
    let path = dir_chain_logs()
        .find(format!("{subject}.fallback_logger.json"))
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;

    std::fs::File::open(path)
}
