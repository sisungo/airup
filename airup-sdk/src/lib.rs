pub mod error;
pub mod ffi;
pub mod files;
pub mod ipc;
pub mod prelude;
pub mod system;

use duplicate::duplicate_item;
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

#[duplicate_item(
    Name;
    [Connection];
    [BlockingConnection];
)]
#[derive(Debug)]
pub struct Name<'a> {
    path: &'a Path,
    underlying: ipc::Name,
}
#[duplicate_item(
    Name                    async      may_await(code)    send_to(who, blob)           receive_from(who);
    [Connection]            [async]    [code.await]       [who.send(blob).await]       [who.recv().await];
    [BlockingConnection]    []         [code]             [who.send_blocking(blob)]    [who.recv_blocking()];
)]
impl<'a> Name<'a> {
    pub async fn connect(path: &'a Path) -> std::io::Result<Name<'a>> {
        Ok(Self {
            path,
            underlying: may_await([ipc::Name::connect(path)])?,
        })
    }

    pub async fn send_raw(&mut self, msg: &[u8]) -> anyhow::Result<()> {
        send_to([self.underlying], [&msg])
    }

    pub async fn recv_raw(&mut self) -> anyhow::Result<Vec<u8>> {
        receive_from([self.underlying])
    }

    pub async fn invoke<P: Serialize, T: DeserializeOwned>(
        &mut self,
        method: &str,
        params: P,
    ) -> anyhow::Result<Result<T, Error>> {
        let req = Request::new(method, params).unwrap();
        may_await([self.underlying.send(&req)])?;
        Ok(may_await([self.underlying.recv_resp()])?.into_result())
    }

    pub fn path(&self) -> &Path {
        self.path
    }
}
#[duplicate_item(
    Name;
    [Connection];
    [BlockingConnection];
)]
impl<'a> Deref for Name<'a> {
    type Target = ipc::Name;

    fn deref(&self) -> &Self::Target {
        &self.underlying
    }
}
#[duplicate_item(
    Name;
    [Connection];
    [BlockingConnection];
)]
impl<'a> DerefMut for Name<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.underlying
    }
}
