//! # The Airup SDK
//! The Airup SDK provides interface to deal with Airup elements, for example, interacting with the daemon, `airupd`.

pub mod blocking;
pub mod build;
pub mod error;
pub mod files;
pub mod ipc;
pub mod prelude;
pub mod system;
mod util;

#[cfg(feature = "nonblocking")]
pub mod nonblocking;

#[cfg(feature = "ffi")]
pub mod ffi;

pub use error::ApiError as Error;

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
