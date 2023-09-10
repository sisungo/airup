//! Inspection of OS multi-user function.

use caches::{AdaptiveCache, Cache};
use std::sync::{Arc, Mutex, OnceLock};
use sysinfo::{SystemExt, UserExt};

pub type Uid = i64;
pub type Gid = i64;

/// Represents to a user database.
pub struct UserDb {
    entry_cache: AdaptiveCache<String, Arc<UserEntry>>,
    req_cache: AdaptiveCache<Request, String>,
}
impl UserDb {
    /// Creates a new [UserDb] instance.
    pub fn new() -> Self {
        crate::env::sysinfo().write().unwrap().refresh_users_list();
        let entry_cache = AdaptiveCache::new(64).unwrap();
        let req_cache = AdaptiveCache::new(64).unwrap();

        Self {
            entry_cache,
            req_cache,
        }
    }

    /// Finds a [UserEntry] by username.
    pub fn find_user_by_name(&mut self, name: &String) -> Option<Arc<UserEntry>> {
        self.entry_cache.get(name).cloned().or_else(|| {
            self.find_user_by_name_uncached(name.into()).map(|v| {
                let v = Arc::new(v);
                self.entry_cache.put(name.into(), v.clone());
                v
            })
        })
    }

    /// Finds a [UserEntry] by UID.
    pub fn find_user_by_uid(&mut self, uid: Uid) -> Option<Arc<UserEntry>> {
        let req = Request::FindEntryByUid(uid);
        match self.req_cache.get(&req) {
            Some(v) => self.find_user_by_name(v),
            None => match self.find_user_by_uid_uncached(uid) {
                Some(v) => {
                    let v = Arc::new(v);
                    self.entry_cache.put(v.name.clone(), v.clone());
                    self.req_cache.put(req, v.name.clone());
                    Some(v)
                }
                None => None,
            },
        }
    }

    /// Refreshes the user database.
    pub fn refresh(&mut self) {
        crate::env::sysinfo().write().unwrap().refresh_users_list();
        self.entry_cache.purge();
        self.req_cache.purge();
    }

    fn find_user_by_uid_uncached(&self, uid: Uid) -> Option<UserEntry> {
        crate::env::sysinfo()
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
    fn find_user_by_name_uncached(&self, name: String) -> Option<UserEntry> {
        crate::env::sysinfo()
            .read()
            .unwrap()
            .users()
            .iter()
            .find(|u| u.name() == name)
            .map(|u| UserEntry {
                uid: **u.id() as _,
                name,
                gid: *u.group_id() as _,
                groups: u.groups().to_vec(),
            })
    }
}
impl Default for UserDb {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum Request {
    FindEntryByUid(Uid),
}

/// Represents to an entry that contains basic user information.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserEntry {
    pub uid: Uid,
    pub name: String,
    pub gid: Gid,
    pub groups: Vec<String>,
}

/// Returns a reference to the global user database.
#[inline]
pub fn user_db() -> &'static Mutex<UserDb> {
    static USER_DB: OnceLock<Mutex<UserDb>> = OnceLock::new();

    USER_DB.get_or_init(Default::default)
}

/// Finds a user entry by UID.
#[inline]
pub fn find_user_by_uid(uid: Uid) -> Option<Arc<UserEntry>> {
    user_db().lock().unwrap().find_user_by_uid(uid)
}

/// Finds a user entry by username.
#[inline]
pub fn find_user_by_name(name: &String) -> Option<Arc<UserEntry>> {
    user_db().lock().unwrap().find_user_by_name(name)
}

/// Returns the user entry of current user.
#[inline]
pub fn current_user() -> Option<Arc<UserEntry>> {
    find_user_by_uid(current_uid())
}

/// Returns UID of current user.
#[inline]
pub fn current_uid() -> Uid {
    unsafe { libc::getuid() as _ }
}
