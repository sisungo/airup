//! # Airup IPC - Server Implementation

pub mod api;

use crate::app::airupd;
use anyhow::anyhow;
use std::{path::Path, sync::Arc};

/// An instance of the Airup IPC context.
#[derive(Debug)]
pub struct Context {
    api: api::Manager,
}
impl Context {
    /// Creates a new `Context` instance.
    pub fn new() -> Self {
        Self {
            api: api::Manager::new(),
        }
    }
}
impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents to an IPC server.
#[derive(Debug)]
pub struct Server {
    server: airup_sdk::nonblocking::ipc::Server,
}
impl Server {
    /// Creates a new [`Server`] instance.
    pub async fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let server = airup_sdk::nonblocking::ipc::Server::new(path)?;
        airupfx::fs::set_sock_permission(path).await?;

        Ok(Self { server })
    }

    /// Forces to create a new [`Server`] instance.
    pub async fn new_force<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        tokio::fs::remove_file(path).await.ok();

        Self::new(path).await
    }

    /// Starts the server task.
    pub fn start(mut self) {
        tokio::spawn(async move {
            self.run().await;
        });
    }

    /// Runs the server in place.
    pub async fn run(&mut self) {
        loop {
            if let Ok(conn) = self
                .server
                .accept()
                .await
                .inspect_err(|e| tracing::warn!("accept() failed: {}", e))
            {
                Session::new(conn).start();
            }
        }
    }
}

/// Represents to an Airupd IPC session.
#[derive(Debug)]
pub struct Session {
    conn: airup_sdk::nonblocking::ipc::Connection,
    context: Arc<SessionContext>,
}
impl Session {
    /// Creates a new `Session` with the given [airup_sdk::ipc::Connection].
    pub fn new(conn: airup_sdk::nonblocking::ipc::Connection) -> Self {
        Self {
            conn,
            context: Arc::default(),
        }
    }

    /// Starts the session task.
    pub fn start(mut self) {
        tokio::spawn(async move {
            if let Err(err) = self.run().await {
                tracing::debug!("{} disconnected: {}", self.audit_name().await, err);
            }
        });
    }

    /// Runs the session in place.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        tracing::debug!("{} established", self.audit_name().await);
        loop {
            let req = self.conn.recv_req().await?;
            if req.method == "debug.disconnect" {
                break Err(anyhow!("invocation of `debug.disconnect`"));
            }
            let resp = airupd().ipc.api.invoke(self.context.clone(), req).await;
            self.conn.send(&resp).await?;
        }
    }

    /// Returns audit-style name of the IPC session.
    pub async fn audit_name(&self) -> String {
        let cred = self.conn.as_ref().peer_cred().ok();
        let uid = cred
            .as_ref()
            .map(|x| x.uid().to_string())
            .unwrap_or_else(|| "?".into());
        let gid = cred
            .as_ref()
            .map(|x| x.gid().to_string())
            .unwrap_or_else(|| "?".into());
        let pid = cred
            .as_ref()
            .and_then(|x| x.pid())
            .map(|x| x.to_string())
            .unwrap_or_else(|| "?".into());
        format!("ipc_session(uid={}, gid={}, pid={})", uid, gid, pid)
    }
}

/// Represents to an Airupd IPC session context.
#[derive(Debug, Default)]
pub struct SessionContext;
