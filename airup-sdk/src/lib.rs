pub mod error;
pub mod ffi;
pub mod files;
pub mod ipc;
pub mod prelude;
pub mod system;

pub use error::ApiError as Error;
use serde::{de::DeserializeOwned, ser::Serialize};

use ipc::Request;
use std::{
    ops::{Deref, DerefMut},
    path::Path,
    sync::OnceLock,
};

/// Returns default path of Airup's IPC socket.
pub fn socket_path() -> &'static Path {
    static SOCKET_PATH: OnceLock<&'static Path> = OnceLock::new();

    SOCKET_PATH.get_or_init(|| {
        Box::leak(
            airupfx::config::build_manifest()
                .runtime_dir
                .join("airupd.sock")
                .into(),
        )
    })
}

#[derive(Debug)]
pub struct Connection<'a> {
    path: &'a Path,
    underlying: ipc::Connection,
}
impl<'a> Connection<'a> {
    pub async fn connect(path: &'a Path) -> std::io::Result<Connection<'a>> {
        Ok(Self {
            path,
            underlying: ipc::Connection::connect(path).await?,
        })
    }

    pub async fn reconnect(&mut self) -> std::io::Result<()> {
        let new_connection = ipc::Connection::connect(self.path).await?;
        self.underlying = new_connection;

        Ok(())
    }

    pub async fn send_raw(&mut self, msg: &[u8]) -> anyhow::Result<()> {
        (*self.underlying).send(msg).await
    }

    pub async fn recv_raw(&mut self) -> anyhow::Result<Vec<u8>> {
        (*self.underlying).recv().await
    }

    pub async fn invoke<P: Serialize, T: DeserializeOwned>(
        &mut self,
        method: &str,
        params: P,
    ) -> anyhow::Result<Result<T, Error>> {
        let req = Request::new(method, params).unwrap();
        self.send(&req).await?;
        Ok(self.recv_resp().await?.into_result())
    }

    pub fn path(&self) -> &Path {
        self.path
    }
}
impl<'a> Deref for Connection<'a> {
    type Target = ipc::Connection;

    fn deref(&self) -> &Self::Target {
        &self.underlying
    }
}
impl<'a> DerefMut for Connection<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.underlying
    }
}
