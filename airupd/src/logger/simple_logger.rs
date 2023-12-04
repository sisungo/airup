//! A simple logger.

use super::Logger;
use airup_sdk::system::LogRecord;
use rev_lines::RevLines;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Default)]
pub struct SimpleLogger {}
impl SimpleLogger {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn open_subject_append(&self, subject: &str) -> std::io::Result<tokio::fs::File> {
        crate::app::airupd()
            .storage
            .logs
            .open_append(&format!("{subject}.simple_logger.json"))
            .await
    }

    pub async fn open_subject_read(&self, subject: &str) -> std::io::Result<tokio::fs::File> {
        crate::app::airupd()
            .storage
            .logs
            .open_read(&format!("{subject}.simple_logger.json"))
            .await
    }
}
#[async_trait::async_trait]
impl Logger for SimpleLogger {
    async fn write(&mut self, subject: &str, module: &str, msg: &[u8]) -> anyhow::Result<()> {
        let mut appender = self.open_subject_append(subject).await?;
        let timestamp = airupfx::time::timestamp_ms();

        for m in msg.split(|x| b"\r\n\0".contains(x)) {
            let record = LogRecord {
                timestamp,
                module: module.to_owned(),
                message: String::from_utf8_lossy(m).into_owned(),
            };
            appender
                .write_all(serde_json::to_string(&record)?.as_bytes())
                .await?;
            appender.write_all(&b"\n"[..]).await?;
        }

        Ok(())
    }

    async fn tail(&mut self, subject: &str, n: usize) -> anyhow::Result<Vec<LogRecord>> {
        let reader = self.open_subject_read(subject).await?.into_std().await;
        tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<LogRecord>> {
            let mut result = Vec::with_capacity(n);
            for line in RevLines::new(reader).take(n) {
                result.push(serde_json::from_str(&line?)?);
            }
            Ok(result)
        })
        .await
        .unwrap()
    }
}
