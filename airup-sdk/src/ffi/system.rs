use crate::blocking::system::ConnectionExt;
use libc::{c_char, c_int};
use std::ffi::CStr;

/// # Safety
/// The caller must guarantee the pointer `name` is valid.
#[no_mangle]
pub unsafe extern "C" fn airup_start_service(
    conn: &mut crate::blocking::Connection,
    name: *const c_char,
) -> c_int {
    super::api_function(|| conn.start_service(&CStr::from_ptr(name).to_string_lossy()))
}

/// # Safety
/// The caller must guarantee the pointer `name` is valid.
#[no_mangle]
pub unsafe extern "C" fn airup_stop_service(
    conn: &mut crate::blocking::Connection,
    name: *const c_char,
) -> c_int {
    super::api_function(|| conn.stop_service(&CStr::from_ptr(name).to_string_lossy()))
}
