//! Inspection and manipulation of the operating system's multi-user function.

use once_cell::sync::Lazy;
use std::sync::RwLock;
use sysinfo::{Uid, User};

static USERS: Lazy<RwLock<sysinfo::Users>> =
    Lazy::new(|| sysinfo::Users::new_with_refreshed_list().into());
static CACHE: Lazy<mini_moka::sync::Cache<Request, Option<usize>, ahash::RandomState>> =
    Lazy::new(|| {
        mini_moka::sync::Cache::builder()
            .initial_capacity(4)
            .max_capacity(16)
            .build_with_hasher(ahash::RandomState::default())
    });

/// Refreshes users database.
pub fn refresh() {
    let mut users = USERS.write().unwrap();
    CACHE.invalidate_all();
    users.refresh_list();
}

/// Finds a user entry by UID.
pub fn with_user_by_id<F: FnOnce(&User) -> T, T>(uid: &Uid, f: F) -> Option<T> {
    let users = USERS.read().unwrap();
    let req = Request::FindUserById(uid.clone());
    let numeric = CACHE.get(&req).unwrap_or_else(|| {
        let value = users
            .iter()
            .enumerate()
            .find(|(_, u)| u.id() == uid)
            .map(|x| x.0);
        CACHE.insert(req, value);
        value
    });
    numeric.map(|i| f(users.get(i).unwrap()))
}

/// Finds a user entry by username.
pub fn with_user_by_name<F: FnOnce(&User) -> T, T>(name: &str, f: F) -> Option<T> {
    let users = USERS.read().unwrap();
    let req = Request::FindUserByName(name.into());
    let numeric = CACHE.get(&req).unwrap_or_else(|| {
        let value = users
            .iter()
            .enumerate()
            .find(|(_, u)| u.name() == name)
            .map(|x| x.0);
        CACHE.insert(req, value);
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
    crate::sys::env::current_uid()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Request {
    FindUserById(Uid),
    FindUserByName(String),
}
