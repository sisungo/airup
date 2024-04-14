//! # The Airup SDK
//! The Airup SDK provides interface to deal with Airup elements, for example, interacting with the daemon, `airupd`.

pub mod build;
pub mod debug;
pub mod error;
pub mod extapi;
pub mod extension;
pub mod files;
pub mod info;
pub mod ipc;
pub mod prelude;
pub mod system;

#[allow(unused)]
mod util;

#[cfg(feature = "nonblocking")]
pub mod nonblocking;

#[cfg(feature = "ffi")]
pub mod ffi;

#[cfg(feature = "blocking")]
pub mod blocking;

pub use error::ApiError as Error;

use serde::{de::DeserializeOwned, Serialize};
use std::{
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
                .unwrap_or_else(|_| build::manifest().runtime_dir.join("airupd.sock"))
                .into(),
        )
    })
}

/// A trait that unifies `async` and `non-async` connections.
pub trait Connection {
    /// Return type of the [`Connection::invoke`] method.
    type Invoke<'a, T: 'a>
    where
        Self: 'a;

    /// Invokes specified method with given parameters on the connection, then wait for a response.
    fn invoke<'a, P: Serialize + Send + 'a, T: DeserializeOwned + 'a>(
        &'a mut self,
        method: &'a str,
        params: P,
    ) -> Self::Invoke<'a, T>;
}
