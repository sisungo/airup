//! Inspection and manipulation of the process's environment.

pub mod users;

pub use users::{current_uid, with_current_user, with_user_by_id, with_user_by_name};

use std::{
    ffi::{OsStr, OsString},
    sync::{OnceLock, RwLock},
};
use sysinfo::SystemExt;

/// Sets environment variables in the iterator for the currently running process, removing environment variables with value
/// `None`.
#[inline]
pub fn set_vars<I: IntoIterator<Item = (K, Option<V>)>, K: Into<OsString>, V: Into<OsString>>(
    iter: I,
) {
    iter.into_iter().for_each(|(k, v)| match v {
        Some(x) => std::env::set_var(k.into(), x.into()),
        None => std::env::remove_var(k.into()),
    });
}

/// Fetches the environment variable key from the current process, then removes the environment variable from the environment
/// of current process.
///
/// ## Panics
/// This function may panic if key is empty, contains an ASCII equals sign '=' or the NUL character '\0', or when value contains
/// the NUL character.
#[inline]
pub fn take_var<K: AsRef<OsStr>>(key: K) -> Result<String, std::env::VarError> {
    let value = std::env::var(key.as_ref())?;
    std::env::remove_var(key);
    Ok(value)
}

/// Refreshes the environmental database.
#[inline]
pub async fn refresh() {
    users::refresh().await;
}

/// Returns a reference to the global unique locked [sysinfo::System] instance.
#[inline]
fn sysinfo() -> &'static RwLock<sysinfo::System> {
    static SYSINFO: OnceLock<RwLock<sysinfo::System>> = OnceLock::new();

    SYSINFO.get_or_init(|| RwLock::new(sysinfo::System::default()))
}

/// Returns host name of the machine currently running the process.
#[inline]
pub fn host_name() -> Option<String> {
    sysinfo().read().unwrap().host_name()
}
