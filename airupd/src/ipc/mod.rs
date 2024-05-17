//! # Airup IPC - Server Implementation

pub mod api;

use crate::app::airupd;
use airup_sdk::ipc::Request;
use anyhow::anyhow;
use std::path::PathBuf;
use tokio::sync::broadcast;

/// An instance of the Airup IPC context.
#[derive(Debug)]
pub struct Context {
    api: api::Manager,
    reload: broadcast::Sender<()>,
}
impl Context {
    /// Creates a new `Context` instance.
    pub fn new() -> Self {
        Self {
            api: api::Manager::new(),
            reload: broadcast::channel(1).0,
        }
    }

    pub fn reload(&self) {
        _ = self.reload.send(());
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
    path: PathBuf,
    server: airup_sdk::nonblocking::ipc::Server,
}
impl Server {
    /// Creates a new [`Server`] instance.
    pub async fn new<P: Into<PathBuf>>(path: P) -> anyhow::Result<Self> {
        let path = path.into();
        let server = airup_sdk::nonblocking::ipc::Server::new(&path)?;
        airupfx::fs::set_permission(&path, airupfx::fs::Permission::Socket).await?;

        Ok(Self { path, server })
    }

    /// Forces to create a new [`Server`] instance.
    pub async fn new_force<P: Into<PathBuf>>(path: P) -> anyhow::Result<Self> {
        let path = path.into();
        _ = tokio::fs::remove_file(&path).await;

        Self::new(path).await
    }

    /// Starts the server task.
    pub fn start(mut self) {
        tokio::spawn(async move {
            self.run().await;
        });
    }

    /// Reloads the server.
    async fn reload(&mut self) -> anyhow::Result<()> {
        let newer = Self::new_force(&self.path).await?;
        *self = newer;
        Ok(())
    }

    /// Runs the server in place.
    async fn run(&mut self) {
        let mut reload = airupd().ipc.reload.subscribe();

        loop {
            tokio::select! {
                Ok(()) = reload.recv() => {
                    _ = self.reload().await;
                },
                Ok(conn) = self.server.accept() => {
                    Session::new(conn).start();
                },
            };
        }
    }
}

/// Represents to an Airupd IPC session.
#[derive(Debug)]
pub struct Session {
    conn: airup_sdk::nonblocking::ipc::Connection,
}
impl Session {
    /// Constructs a new [`Session`] instance with connection `conn`.
    fn new(conn: airup_sdk::nonblocking::ipc::Connection) -> Self {
        Self { conn }
    }

    /// Starts the session task.
    fn start(mut self) {
        tokio::spawn(async move {
            if let Err(err) = self.run().await {
                tracing::debug!("{} disconnected: {}", self.audit_name().await, err);
            }
        });
    }

    /// Runs the session in place.
    async fn run(&mut self) -> anyhow::Result<()> {
        tracing::debug!("{} established", self.audit_name().await);
        loop {
            let req = self.conn.recv_req().await?;
            if req.method == "debug.disconnect" {
                break Err(anyhow!("invocation of `debug.disconnect`"));
            }
            let resp = match req.method.strip_prefix("extapi.") {
                Some(method) => {
                    airupd()
                        .extensions
                        .rpc_invoke(Request::new::<&str, ciborium::Value, _>(method, req.params))
                        .await
                }
                None => airupd().ipc.api.invoke(req).await,
            };
            self.conn.send(&resp).await?;
        }
    }

    /// Returns audit-style name of the IPC session.
    async fn audit_name(&self) -> String {
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
