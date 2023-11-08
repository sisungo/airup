//! Signal handling for Microsoft Windows.

use std::future::Future;

/// Software termination signal.
pub const SIGTERM: i32 = libc::SIGTERM;

/// Kills the process.
pub const SIGKILL: i32 = 9;

/// Registers a signal handler.
///
/// # Errors
/// An `Err(_)` is returned if the underlying OS function failed.
pub fn signal<
    F: FnMut(i32) -> T + Send + Sync + 'static,
    T: Future<Output = ()> + Send + 'static,
>(
    signum: i32,
    mut op: F,
) -> anyhow::Result<()> {
    todo!()
}
