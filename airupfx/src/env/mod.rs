//! Inspection and manipulation of the process's environment.

pub mod users;

pub use users::{current_uid, with_current_user, with_user_by_id, with_user_by_name};

use once_cell::sync::Lazy;
use std::{
    ffi::{OsStr, OsString},
    sync::RwLock,
};
use sysinfo::SystemExt;

static SYSINFO: Lazy<RwLock<sysinfo::System>> = Lazy::new(|| RwLock::new(sysinfo::System::new()));

/// Sets environment variables in the iterator for the currently running process, removing environment variables with value
/// `None`.
///
/// # Panics
/// This function may panic if any of the keys is empty, contains an ASCII equals sign '=' or the NUL character '\0', or when
/// the value contains the NUL character.
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
/// # Panics
/// This function may panic if key is empty, contains an ASCII equals sign '=' or the NUL character '\0', or when value contains
/// the NUL character.
///
/// # Errors
/// An `Err(_)` is returned if the specific variable is not existing.
#[inline]
pub fn take_var<K: AsRef<OsStr>>(key: K) -> Result<String, std::env::VarError> {
    let value = std::env::var(key.as_ref())?;
    std::env::remove_var(key);
    Ok(value)
}

/// Refreshes the environmental database.
#[inline]
pub async fn refresh() {
    users::refresh();
}

/// Returns host name of the machine currently running the process.
#[inline]
pub fn host_name() -> Option<String> {
    SYSINFO.read().unwrap().host_name()
}
