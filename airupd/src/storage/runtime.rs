//! Represents to Airup's runtime directory.

use crate::ipc;
use anyhow::anyhow;
use std::path::PathBuf;

/// Main navigator of Airup's runtime directory.
#[derive(Debug)]
pub struct Runtime {
    base_dir: PathBuf,
}
impl Runtime {
    /// Creates a new [`Runtime`] instance.
    pub async fn new() -> Self {
        let base_dir = airup_sdk::build::manifest().runtime_dir.clone();
        tokio::fs::create_dir_all(&base_dir).await.ok();

        Self { base_dir }
    }

    /// Locks airup data.
    pub async fn lock(&self) -> anyhow::Result<Lock> {
        Lock::new(self.base_dir.join("airupd.lock")).await
    }

    /// Creates an IPC server.
    pub async fn ipc_server(&self) -> anyhow::Result<ipc::Server> {
        let socket_path = self.base_dir.join("airupd.sock");
        std::env::set_var("AIRUP_SOCK", &socket_path);
        ipc::Server::new_force(&socket_path).await
    }
}

/// Represents to a lock file.
#[derive(Debug)]
pub struct Lock(PathBuf);
impl Lock {
    /// Creates an owned `Lock` instance for specified path.
    pub async fn new(path: PathBuf) -> anyhow::Result<Self> {
        if *airupfx::process::ID != 1 && tokio::fs::try_exists(&path).await.unwrap_or(true) {
            return Err(anyhow!("already locked"));
        }

        tokio::fs::write(&path, airupfx::process::ID.to_string().as_bytes()).await?;

        Ok(Self(path))
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).ok();
    }
}
