use super::util::*;
use crate::system::ConnectionExt;
use libc::{c_char, c_int};
use std::ffi::CStr;

/// # Safety
/// The caller must guarantee the pointer `name` is valid.
#[no_mangle]
pub unsafe extern "C" fn airup_start_service(
    conn: &mut crate::blocking::Connection,
    name: *const c_char,
) -> c_int {
    api_function(|| Ok(conn.start_service(&CStr::from_ptr(name).to_string_lossy())?))
}

/// # Safety
/// The caller must guarantee the pointer `name` is valid.
#[no_mangle]
pub unsafe extern "C" fn airup_stop_service(
    conn: &mut crate::blocking::Connection,
    name: *const c_char,
) -> c_int {
    api_function(|| Ok(conn.stop_service(&CStr::from_ptr(name).to_string_lossy())?))
}

/// # Safety
/// The caller must guarantee the pointer `event` is valid.
#[no_mangle]
pub unsafe extern "C" fn airup_trigger_event(
    conn: &mut crate::blocking::Connection,
    event: *const c_char,
) -> c_int {
    api_function(|| Ok(conn.trigger_event(&CStr::from_ptr(event).to_string_lossy())?))
}
