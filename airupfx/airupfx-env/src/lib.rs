//! Inspection and manipulation of the process's environment.

pub mod users;

pub use users::{current_uid, with_current_user, with_user_by_id, with_user_by_name};

use std::{
    ffi::{OsStr, OsString},
    path::Path,
    sync::RwLock,
};

static INSTANCE_NAME: RwLock<String> = RwLock::new(String::new());

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

/// Sets instance name of the process.
#[inline]
pub fn set_instance_name(name: String) {
    *INSTANCE_NAME.write().unwrap() = name;
}

/// Returns instance name of the process. This is default to the machine's host name, and can be set via [`set_instance_name`].
#[inline]
pub fn instance_name() -> String {
    let global = INSTANCE_NAME.read().unwrap();
    if global.is_empty() {
        sysinfo::System::host_name().unwrap_or_else(|| String::from("localhost"))
    } else {
        global.clone()
    }
}

pub async fn setup_stdio(path: &Path) -> std::io::Result<()> {
    cfg_if::cfg_if! {
        if #[cfg(target_family = "unix")] {
            use std::os::unix::io::AsRawFd;

            loop {
                let file = tokio::fs::File::options()
                    .read(true)
                    .write(true)
                    .open(path)
                    .await?;
                if file.as_raw_fd() >= 3 {
                    break Ok(());
                } else {
                    std::mem::forget(file);
                }
    }
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn take_var() {
        std::env::set_var("magic", "1");
        assert!(matches!(std::env::var("magic").as_deref(), Ok("1")));
        assert!(matches!(crate::take_var("magic").as_deref(), Ok("1")));
        assert!(matches!(std::env::var("magic").as_deref(), Err(_)));
    }

    #[test]
    fn instance_name() {
        let x = crate::instance_name();
        crate::set_instance_name(format!("{x}-testing"));
        let y = crate::instance_name();
        assert_ne!(x, y);
    }
}
