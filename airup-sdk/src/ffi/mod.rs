//! # Airup SDK for C
//! This module contains Airup SDK for the C programming language.

pub mod error;
pub mod info;
pub mod system;
pub mod util;

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
pub extern "C" fn airup_build_manifest() -> *const libc::c_char {
    static VALUE: std::sync::OnceLock<std::ffi::CString> = std::sync::OnceLock::new();

    let default = || {
        std::ffi::CString::new(include_str!("../../../build_manifest.json"))
            .expect("`build_manifest.json` should never contain NUL")
    };

    VALUE.get_or_init(default).as_ptr()
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
