//! Inspection and manipulation of the operating system's multi-user function.

use std::{
    ffi::{CStr, CString},
    mem::MaybeUninit,
};

/// An entry in the system's user database.
#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
    pub uid: libc::uid_t,
    pub gid: libc::gid_t,
    pub groups: Vec<libc::gid_t>,
}

/// Finds a user entry by UID.
pub fn user_by_id(uid: libc::uid_t) -> Option<User> {
    with_passwd_entry(|pwd, buf, len, result| unsafe {
        libc::getpwuid_r(uid, pwd, buf, len, result)
    })
}

/// Finds a user entry by username.
pub fn user_by_name(name: &str) -> Option<User> {
    let name = CString::new(name).ok()?;
    with_passwd_entry(|pwd, buf, len, result| unsafe {
        libc::getpwnam_r(name.as_ptr(), pwd, buf, len, result)
    })
}

/// Returns the user entry of current user.
pub fn current_user() -> Option<User> {
    user_by_id(current_uid())
}

/// Returns UID of current user.
pub fn current_uid() -> libc::uid_t {
    cfg_if::cfg_if! {
        if #[cfg(target_family = "unix")] {
            unsafe { libc::getuid() }
        } else {
            std::compile_error!("This target is not supported by `Airup` yet. Consider opening an issue at https://github.com/sisungo/airup/issues?");
        }
    }
}

/// The size to try first when the system does not report one.
const INITIAL_BUFFER_SIZE: usize = 1024;

/// The maximum buffer size we are willing to allocate for a single lookup.
const MAX_BUFFER_SIZE: usize = 1 << 20;

/// On most platforms `getgrouplist(3)` takes `int *` for the group list, while the group IDs
/// themselves are `gid_t`. Linux is the odd one out and uses `gid_t *`.
#[cfg(any(target_os = "linux", target_os = "android"))]
type GroupListElement = libc::gid_t;
#[cfg(not(any(target_os = "linux", target_os = "android")))]
type GroupListElement = libc::c_int;

/// Queries a `passwd` entry with the specified reentrant lookup function.
fn with_passwd_entry<F>(mut query: F) -> Option<User>
where
    F: FnMut(*mut libc::passwd, *mut libc::c_char, usize, *mut *mut libc::passwd) -> libc::c_int,
{
    let mut size = match unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) } {
        size if size > 0 => size as usize,
        _ => INITIAL_BUFFER_SIZE,
    };
    let mut buf = vec![0 as libc::c_char; size];

    loop {
        let mut pwd = MaybeUninit::<libc::passwd>::uninit();
        let mut result = std::ptr::null_mut();
        let error = query(pwd.as_mut_ptr(), buf.as_mut_ptr(), buf.len(), &mut result);
        if error == libc::ERANGE && size < MAX_BUFFER_SIZE {
            size = (size * 2).min(MAX_BUFFER_SIZE);
            buf.resize(size, 0);
            continue;
        }
        if error != 0 || result.is_null() {
            return None;
        }

        // SAFETY: `result` points to `pwd`, which was filled in by the query above. Its string
        // fields point into `buf`, which outlives this block.
        let pwd = unsafe { pwd.assume_init() };
        let name = unsafe { CStr::from_ptr(pwd.pw_name) };
        return Some(User {
            username: name.to_string_lossy().into_owned(),
            uid: pwd.pw_uid,
            gid: pwd.pw_gid,
            groups: groups_of(name, pwd.pw_gid),
        });
    }
}

/// Returns the group list of the user with the specific name, including its primary group.
fn groups_of(name: &CStr, gid: libc::gid_t) -> Vec<libc::gid_t> {
    let mut capacity = 16;

    loop {
        let mut groups = vec![0 as GroupListElement; capacity];
        let mut count = capacity as libc::c_int;
        let error =
            unsafe { libc::getgrouplist(name.as_ptr(), gid as _, groups.as_mut_ptr(), &mut count) };
        if error >= 0 {
            return groups
                .into_iter()
                .take(count.max(0) as usize)
                .map(|x| x as libc::gid_t)
                .collect();
        }

        // `getgrouplist` reports the required buffer size on failure.
        if count <= capacity as libc::c_int {
            return Vec::new();
        }
        capacity = count as usize;
    }
}
