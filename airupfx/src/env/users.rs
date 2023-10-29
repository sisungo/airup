use once_cell::sync::Lazy;
use std::sync::RwLock;
use sysinfo::{SystemExt, Uid, User, UserExt};

static USERS: Lazy<RwLock<sysinfo::System>> = Lazy::new(|| {
    let mut instance = sysinfo::System::new();
    instance.refresh_users_list();
    instance.into()
});
static CACHE: Lazy<mini_moka::sync::Cache<Request, Option<usize>, ahash::RandomState>> =
    Lazy::new(|| {
        mini_moka::sync::Cache::builder()
            .initial_capacity(4)
            .max_capacity(64)
            .build_with_hasher(ahash::RandomState::default())
    });

/// Refreshes users database.
pub fn refresh() {
    USERS.write().unwrap().refresh_users_list();
    CACHE.invalidate_all();
}

/// Finds a user entry by UID.
pub fn with_user_by_id<F: FnOnce(&User) -> T, T>(uid: &Uid, f: F) -> Option<T> {
    let users = USERS.read().unwrap();
    let req = Request::FindUserById(uid.clone());
    let numeric = CACHE.get(&req).unwrap_or_else(|| {
        let value = users
            .users()
            .iter()
            .enumerate()
            .find(|(_, u)| u.id() == uid)
            .map(|x| x.0);
        CACHE.insert(req, value);
        value
    });
    numeric.map(|i| f(users.users().get(i).unwrap()))
}

/// Finds a user entry by username.
pub fn with_user_by_name<F: FnOnce(&User) -> T, T>(name: &str, f: F) -> Option<T> {
    let users = USERS.read().unwrap();
    let req = Request::FindUserByName(name.into());
    let numeric = CACHE.get(&req).unwrap_or_else(|| {
        let value = users
            .users()
            .iter()
            .enumerate()
            .find(|(_, u)| u.name() == name)
            .map(|x| x.0);
        CACHE.insert(req, value);
        value
    });
    numeric.map(|i| f(users.users().get(i).unwrap()))
}

/// Returns the user entry of current user.
pub fn with_current_user<F: FnOnce(&User) -> T, T>(f: F) -> Option<T> {
    with_user_by_id(&current_uid(), f)
}

/// Returns UID of current user.
pub fn current_uid() -> Uid {
    Uid::try_from(unsafe { libc::getuid() as usize }).unwrap()
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Request {
    FindUserById(Uid),
    FindUserByName(String),
}
