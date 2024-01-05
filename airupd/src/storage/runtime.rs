//! Represents to Airup's runtime directory.

use crate::ipc;
use airupfx::fs::Lock;
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
    pub async fn lock(&self) -> std::io::Result<Lock> {
        Lock::new(self.base_dir.join("airupd.lock")).await
    }

    /// Creates an IPC server.
    pub async fn ipc_server(&self) -> anyhow::Result<ipc::Server> {
        let socket_path = self.base_dir.join("airupd.sock");
        std::env::set_var("AIRUP_SOCK", &socket_path);
        ipc::Server::new_force(&socket_path).await
    }
}
