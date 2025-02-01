//! # Airup IPC - Server Implementation

pub mod api;

use crate::app::airupd;
use airup_sdk::rpc::Request;
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

/// Represents to an RPC server.
#[derive(Debug)]
pub struct Server {
    path: Option<PathBuf>,
    server: airup_sdk::nonblocking::rpc::Server,
}
impl Server {
    /// Creates a new [`Server`] instance.
    pub async fn with_path<P: Into<PathBuf>>(path: P) -> anyhow::Result<Self> {
        let path = path.into();
        let server = airup_sdk::nonblocking::rpc::Server::new(&path)?;
        airupfx::fs::set_permission(&path, airupfx::fs::Permission::Socket).await?;

        Ok(Self {
            path: Some(path),
            server,
        })
    }

    /// Forces to create a new [`Server`] instance.
    pub async fn with_path_force<P: Into<PathBuf>>(path: P) -> anyhow::Result<Self> {
        let path = path.into();
        _ = tokio::fs::remove_file(&path).await;

        Self::with_path(path).await
    }

    /// Starts the server task.
    pub fn start(mut self) {
        tokio::spawn(async move {
            self.run().await;
        });
    }

    /// Reloads the server.
    async fn reload(&mut self) -> anyhow::Result<()> {
        if let Some(path) = self.path.as_ref() {
            let newer = Self::with_path_force(path).await?;
            *self = newer;
        }
        Ok(())
    }

    /// Runs the server in place.
    async fn run(&mut self) {
        let mut reload = airupd().rpc.reload.subscribe();

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
    conn: airup_sdk::nonblocking::rpc::Connection,
}
impl Session {
    /// Constructs a new [`Session`] instance with connection `conn`.
    fn new(conn: airup_sdk::nonblocking::rpc::Connection) -> Self {
        Self { conn }
    }

    /// Starts the session task.
    fn start(self) {
        tokio::spawn(async move {
            _ = self.run().await;
        });
    }

    /// Runs the session in place.
    async fn run(mut self) -> anyhow::Result<()> {
        loop {
            let req = self.conn.recv_req().await?;
            if req.method.strip_prefix("session.").is_some() {
                api::session::invoke(self, req).await;
                return Ok(());
            }
            let resp = match req.method.strip_prefix("extapi.") {
                Some(method) => airupd()
                    .extensions
                    .rpc_invoke(Request::new::<&str, ciborium::Value, _>(method, req.params))
                    .await
                    .unwrap(),
                None => airupd().rpc.api.invoke(req).await,
            };
            self.conn.send(&resp).await?;
        }
    }
}
