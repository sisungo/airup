//! # Airup SDK for C
//! This module contains Airup SDK for the C programming language.

pub mod error;
pub mod system;

pub fn alloc_c_string(rs: &str) -> *mut libc::c_char {
    std::ffi::CString::new(rs.replace('\0', "\u{FFFD}"))
        .expect("filtered string should never contain nul")
        .into_raw() as _
}

/// # Safety
/// Calling this on the same pointer more than once may cause **double-free** problem.
pub unsafe fn dealloc_c_string(p: *mut libc::c_char) {
    drop(std::ffi::CString::from_raw(p as *mut _));
}

/// # Safety
/// The caller must guarantee the pointer is valid.
pub unsafe fn duplicate_c_string(p: *const libc::c_char) -> *mut libc::c_char {
    std::ffi::CString::from(std::ffi::CStr::from_ptr(p)).into_raw() as _
}

pub fn alloc<T>(value: T) -> *mut libc::c_void {
    Box::into_raw(Box::new(value)) as _
}

/// # Safety
/// Calling this on the same pointer more than once may cause **double-free** problem.
pub unsafe fn dealloc<T>(ptr: *mut T) {
    drop(Box::from_raw(ptr));
}

/// # Safety
/// The caller must guarantee the pointer `path` is valid.
#[no_mangle]
pub unsafe extern "C" fn airup_connect(
    path: *const libc::c_char,
) -> Option<Box<super::blocking::Connection>> {
    let path = std::ffi::CStr::from_ptr(path);
    match super::blocking::Connection::connect(&*path.to_string_lossy()) {
        Ok(conn) => Some(Box::new(conn)),
        Err(err) => {
            error::set_last_error(err.into());
            None
        }
    }
}

#[no_mangle]
pub extern "C" fn airup_disconnect(conn: Box<super::blocking::Connection>) {
    drop(conn);
}

#[no_mangle]
pub extern "C" fn airup_default_path() -> *const libc::c_char {
    static VALUE: std::sync::OnceLock<std::ffi::CString> = std::sync::OnceLock::new();

    let default = || {
        std::ffi::CString::new(&*crate::socket_path().to_string_lossy())
            .expect("`AIRUP_SOCK` should never contain NUL")
    };

    VALUE.get_or_init(default).as_ptr()
}

fn api_function<F: FnOnce() -> anyhow::Result<Result<(), crate::Error>>>(f: F) -> libc::c_int {
    match f() {
        Ok(Ok(())) => 0,
        Ok(Err(err)) => {
            error::set_last_error(err.into());
            -1
        }
        Err(err) => {
            let err = crate::Error::Io {
                message: err.to_string(),
            };
            error::set_last_error(err.into());
            -1
        }
    }
}
