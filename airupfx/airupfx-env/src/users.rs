//! Inspection and manipulation of the operating system's multi-user function.

use std::sync::{OnceLock, RwLock};
use sysinfo::{Uid, User};

fn sysinfo_users() -> &'static RwLock<sysinfo::Users> {
    static USERS: OnceLock<RwLock<sysinfo::Users>> = OnceLock::new();
    USERS.get_or_init(|| sysinfo::Users::new_with_refreshed_list().into())
}

/// Refreshes users database.
pub fn refresh() {
    sysinfo_users().write().unwrap().refresh();
}

/// Finds a user entry by UID.
pub fn with_user_by_id<F: FnOnce(&User) -> T, T>(uid: &Uid, f: F) -> Option<T> {
    Some(f(sysinfo_users()
        .read()
        .unwrap()
        .iter()
        .find(|u| u.id() == uid)?))
}

/// Finds a user entry by username.
pub fn with_user_by_name<F: FnOnce(&User) -> T, T>(name: &str, f: F) -> Option<T> {
    Some(f(sysinfo_users()
        .read()
        .unwrap()
        .iter()
        .find(|u| u.name() == name)?))
}

/// Returns the user entry of current user.
pub fn with_current_user<F: FnOnce(&User) -> T, T>(f: F) -> Option<T> {
    with_user_by_id(&current_uid(), f)
}

/// Returns UID of current user.
pub fn current_uid() -> Uid {
    cfg_if::cfg_if! {
        if #[cfg(target_family = "unix")] {
            Uid::try_from(unsafe { libc::getuid() } as usize).unwrap()
        } else {
            std::compile_error!("This target is not supported by `Airup` yet. Consider opening an issue at https://github.com/sisungo/airup/issues?");
        }
    }
}
