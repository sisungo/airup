//! Inspection and manipulation of the process's environment.

use std::{
    ffi::{OsStr, OsString},
    sync::{
        atomic::{AtomicBool, Ordering},
        OnceLock, RwLock,
    },
};
use sysinfo::{SystemExt, UserExt};

pub type Uid = i64;
pub type Gid = i64;

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

/// Returns a reference to the global unique locked [sysinfo::System] instance.
#[inline]
fn sysinfo() -> &'static RwLock<sysinfo::System> {
    static SYSINFO: OnceLock<RwLock<sysinfo::System>> = OnceLock::new();

    SYSINFO.get_or_init(|| RwLock::new(sysinfo::System::default()))
}

#[inline]
fn users() -> &'static RwLock<sysinfo::System> {
    static INITIALIZED: AtomicBool = AtomicBool::new(false);
    if !INITIALIZED.fetch_or(true, Ordering::SeqCst) {
        sysinfo().write().unwrap().refresh_users_list();
    }
    sysinfo()
}

/// Refreshes users database.
#[inline]
pub fn refresh_users() {
    users().write().unwrap().refresh_users_list()
}

/// Refreshes the environmental database.
#[inline]
pub fn refresh() {
    refresh_users();
}

/// Returns host name of the machine currently running the process.
#[inline]
pub fn host_name() -> Option<String> {
    sysinfo().read().unwrap().host_name()
}

/// Finds a user entry by UID.
#[inline]
pub fn find_user_by_uid(uid: Uid) -> Option<UserEntry> {
    users()
        .read()
        .unwrap()
        .get_user_by_id(&sysinfo::Uid::try_from(uid as usize).ok()?)
        .map(|u| UserEntry {
            uid,
            gid: *u.group_id() as _,
            name: u.name().into(),
            groups: u.groups().to_vec(),
        })
}

/// Finds a user entry by username.
#[inline]
pub fn find_user_by_name(name: &String) -> Option<UserEntry> {
    users()
        .read()
        .unwrap()
        .users()
        .iter()
        .find(|u| u.name() == name)
        .map(|u| UserEntry {
            uid: **u.id() as _,
            name: name.into(),
            gid: *u.group_id() as _,
            groups: u.groups().to_vec(),
        })
}

/// Returns the user entry of current user.
#[inline]
pub fn current_user() -> Option<UserEntry> {
    find_user_by_uid(current_uid())
}

/// Returns UID of current user.
#[inline]
pub fn current_uid() -> Uid {
    unsafe { libc::getuid() as _ }
}

/// Represents to an entry that contains basic user information.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserEntry {
    pub uid: Uid,
    pub name: String,
    pub gid: Gid,
    pub groups: Vec<String>,
}
