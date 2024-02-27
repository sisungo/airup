use super::util::*;
use crate::system::{ConnectionExt, Event};
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
    id: *const c_char,
    payload: *const c_char,
) -> c_int {
    let id = CStr::from_ptr(id).to_string_lossy().into();
    let payload = if payload.is_null() {
        String::new()
    } else {
        CStr::from_ptr(payload).to_string_lossy().into()
    };
    let event = Event::new(id, payload);
    api_function(|| Ok(conn.trigger_event(&event)?))
}
