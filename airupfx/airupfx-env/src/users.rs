//! Inspection and manipulation of the operating system's multi-user function.

use std::sync::{OnceLock, RwLock};
use sysinfo::{Uid, User};

type Cache = quick_cache::sync::Cache<Request, Option<usize>>;

fn sysinfo_users() -> &'static RwLock<sysinfo::Users> {
    static USERS: OnceLock<RwLock<sysinfo::Users>> = OnceLock::new();
    USERS.get_or_init(|| sysinfo::Users::new_with_refreshed_list().into())
}

fn cache() -> &'static Cache {
    static CACHE: OnceLock<Cache> = OnceLock::new();
    CACHE.get_or_init(|| quick_cache::sync::Cache::new(16))
}

/// Refreshes users database.
pub fn refresh() {
    let mut users = sysinfo_users().write().unwrap();
    cache().clear();
    users.refresh_list();
}

/// Finds a user entry by UID.
pub fn with_user_by_id<F: FnOnce(&User) -> T, T>(uid: &Uid, f: F) -> Option<T> {
    let users = sysinfo_users().read().unwrap();
    let req = Request::FindUserById(uid.clone());
    let numeric = cache().get(&req).unwrap_or_else(|| {
        let value = users
            .iter()
            .enumerate()
            .find(|(_, u)| u.id() == uid)
            .map(|x| x.0);
        cache().insert(req, value);
        value
    });
    numeric.map(|i| f(users.get(i).unwrap()))
}

/// Finds a user entry by username.
pub fn with_user_by_name<F: FnOnce(&User) -> T, T>(name: &str, f: F) -> Option<T> {
    let users = sysinfo_users().read().unwrap();
    let req = Request::FindUserByName(name.into());
    let numeric = cache().get(&req).unwrap_or_else(|| {
        let value = users
            .iter()
            .enumerate()
            .find(|(_, u)| u.name() == name)
            .map(|x| x.0);
        cache().insert(req, value);
        value
    });
    numeric.map(|i| f(users.get(i).unwrap()))
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Request {
    FindUserById(Uid),
    FindUserByName(String),
}
