//! # The Airup SDK
//! The Airup SDK provides interface to deal with Airup elements, for example, interacting with the daemon, `airupd`. This
//! cargo project contains code for the SDK in both `Rust` and `C` programming languages.

pub mod error;
pub mod ffi;
pub mod files;
pub mod ipc;
pub mod prelude;
pub mod system;

pub use error::ApiError as Error;
use ipc::Request;
use serde::{de::DeserializeOwned, ser::Serialize};
use std::{
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
    sync::OnceLock,
};

/// Returns default path of Airup's IPC socket.
///
/// If environment `AIRUP_SOCK` was present, returns the value of `AIRUP_SOCK`. Otherwise it returns `$runtime_dir/airupd.sock`,
/// which is related to the compile-time `build_manifest.json`.
pub fn socket_path() -> &'static Path {
    static SOCKET_PATH: OnceLock<&'static Path> = OnceLock::new();

    SOCKET_PATH.get_or_init(|| {
        Box::leak(
            std::env::var("AIRUP_SOCK")
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    airupfx::config::build_manifest()
                        .runtime_dir
                        .join("airupd.sock")
                })
                .into(),
        )
    })
}

#[derive(Debug)]
pub struct Connection {
    underlying: ipc::Connection,
}
impl Connection {
    pub async fn connect<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        Ok(Self {
            underlying: ipc::Connection::connect(path).await?,
        })
    }

    pub async fn send_raw(&mut self, msg: &[u8]) -> anyhow::Result<()> {
        self.underlying.send(&msg).await
    }

    pub async fn recv_raw(&mut self) -> anyhow::Result<Vec<u8>> {
        self.underlying.recv().await
    }

    pub async fn invoke<P: Serialize, T: DeserializeOwned>(
        &mut self,
        method: &str,
        params: P,
    ) -> anyhow::Result<Result<T, Error>> {
        let req = Request::new(method, params).unwrap();
        self.underlying.send(&req).await?;
        Ok(self.underlying.recv_resp().await?.into_result())
    }
}
impl Deref for Connection {
    type Target = ipc::Connection;

    fn deref(&self) -> &Self::Target {
        &self.underlying
    }
}
impl DerefMut for Connection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.underlying
    }
}
