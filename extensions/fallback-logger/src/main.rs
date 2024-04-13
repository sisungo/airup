//! A simple logger.
//!
//! This has some limitations and has bad performance. Being designed as an "fallback choice", the implementation aims to be
//! small.

use airup_sdk::{
    extension::server::{MethodFuture, RpcApi, Server},
    nonblocking::fs::DirChain,
    system::LogRecord,
    Error,
};
use rev_lines::RevLines;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut rpc_api = RpcApi::new();
    rpc_api.add("logger.append".into(), write);
    rpc_api.add("logger.tail".into(), tail);
    Server::new(Arc::new(rpc_api)).run().await.unwrap();
}

pub fn dir_chain_logs() -> DirChain<'static> {
    DirChain::new(&airup_sdk::build::manifest().log_dir)
}

pub async fn open_subject_append(subject: &str) -> std::io::Result<tokio::fs::File> {
    let path = dir_chain_logs()
        .find_or_create(&format!("{subject}.fallback_logger.json"))
        .await?;

    tokio::fs::File::options()
        .append(true)
        .create(true)
        .open(path)
        .await
}

pub async fn open_subject_read(subject: &str) -> std::io::Result<tokio::fs::File> {
    let path = dir_chain_logs()
        .find(&format!("{subject}.fallback_logger.json"))
        .await
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;

    tokio::fs::File::open(path).await
}

#[airupfx::macros::api]
async fn write(subject: String, module: String, msg: Vec<u8>) -> Result<(), Error> {
    let mut appender = open_subject_append(&subject).await.map_err(|x| Error::Io {
        message: x.to_string(),
    })?;
    let timestamp = airupfx::time::timestamp_ms();

    for m in msg.split(|x| b"\r\n\0".contains(x)) {
        let record = LogRecord {
            timestamp,
            module: module.to_owned(),
            message: String::from_utf8_lossy(m).into_owned(),
        };
        appender
            .write_all(serde_json::to_string(&record).unwrap().as_bytes())
            .await
            .map_err(|x| Error::Io {
                message: x.to_string(),
            })?;
        appender
            .write_all(&b"\n"[..])
            .await
            .map_err(|x| Error::Io {
                message: x.to_string(),
            })?;
    }

    Ok(())
}

#[airupfx::macros::api]
async fn tail(subject: String, n: usize) -> Result<Vec<LogRecord>, Error> {
    let reader = open_subject_read(&subject)
        .await
        .map_err(|x| Error::Io {
            message: x.to_string(),
        })?
        .into_std()
        .await;
    tokio::task::spawn_blocking(move || -> Result<Vec<LogRecord>, Error> {
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
    })
    .await
    .unwrap()
}
