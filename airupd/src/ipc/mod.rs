//! # Airup IPC - Server Implementation

#![allow(unstable_name_collisions)]

pub mod api;

use crate::app::airupd;
use airupfx::prelude::*;
use anyhow::anyhow;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

/// An instance of the Airup IPC context.
#[derive(Debug)]
pub struct Context {
    pub api: api::Manager,
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
    path: PathBuf,
    server: airupfx::ipc::Server,
}
impl Server {
    /// Creates a new `Server` instance.
    pub async fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path: PathBuf = path.as_ref().into();
        let server = airupfx::ipc::Server::new(&path)?;

        Ok(Self { path, server })
    }

    /// Forces to create a new `Server` instance.
    pub async fn new_force<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        tokio::fs::remove_file(path).await.ok();

        Self::new(path).await
    }

    /// Resets the server IPC handles.
    ///
    /// This is called when the socket file got lost.
    pub async fn reset_ipc(&mut self) -> anyhow::Result<()> {
        let new_server = airupfx::ipc::Server::new(&self.path)?;
        self.server = new_server;

        Ok(())
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
            if !tokio::fs::try_exists(&self.path).await.unwrap_or_default() {
                self.reset_ipc()
                    .await
                    .inspect_err(|e| tracing::warn!("ipc_reset() failed: {}", e))
                    .ok();
            }
            if let Ok(conn) = self
                .server
                .accept()
                .await
                .inspect_err(|e| tracing::warn!("accept() failed: {}", e))
            {
                Session::new(conn).await.start();
            }
        }
    }
}

/// Represents to an Airupd IPC session.
#[derive(Debug)]
pub struct Session {
    conn: airupfx::ipc::Connection,
    context: Arc<SessionContext>,
}
impl Session {
    /// Creates a new `Session` with the given [airupfx::ipc::Connection].
    pub async fn new(conn: airupfx::ipc::Connection) -> Self {
        let context = Arc::new(SessionContext::with_conn(&conn));
        Self { conn, context }
    }

    /// Starts the session task.
    pub fn start(mut self) {
        tokio::spawn(async move {
            if let Err(err) = self.run().await {
                tracing::info!("{} exited: {}", self.audit_name(), err);
            }
        });
    }

    /// Runs the session in place.
    pub async fn run(&mut self) -> anyhow::Result<()> {
        tracing::info!("{} established", self.audit_name());
        loop {
            let req = self.conn.recv_req().await?;
            if req.method == "debug.disconnect" {
                break Err(anyhow!("invocation of `debug.disconnect`"));
            }
            let resp = airupd().ipc.api.invoke(self.context.clone(), req).await;
            self.conn.send(&resp).await?;
        }
    }

    /// Returns audit-style name of this session in string.
    pub fn audit_name(&self) -> String {
        let null = || "null".into();
        let uid = self
            .context
            .uid
            .as_ref()
            .map(|uid| uid.to_string())
            .unwrap_or_else(null);
        let pid = self
            .context
            .pid
            .map(|pid| pid.to_string())
            .unwrap_or_else(null);

        format!("ipc_session(uid={uid}, pid={pid})")
    }
}

/// Represents to an Airupd IPC session context.
#[derive(Debug)]
pub struct SessionContext {
    pub uid: Option<Uid>,
    pub pid: Option<i64>,
}
impl SessionContext {
    pub fn with_conn(conn: &airupfx::ipc::Connection) -> Self {
        let cred = conn.cred().ok();
        let uid = cred
            .as_ref()
            .map(|c| Uid::try_from(c.uid() as usize).unwrap());
        let pid = cred.as_ref().and_then(|c| c.pid()).map(|x| x as _);

        SessionContext { uid, pid }
    }
}
