//! Inspection and manipulation of the process's environment.

use std::ffi::{OsStr, OsString};

/// Sets environment variables in the iterator for the currently running process, removing environment variables with value
/// `None`.
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
pub fn take_var<K: AsRef<OsStr>>(key: K) -> Result<String, std::env::VarError> {
    let value = std::env::var(key.as_ref())?;
    std::env::remove_var(key);
    Ok(value)
}
