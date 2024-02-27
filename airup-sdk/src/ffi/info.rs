use super::util::*;
use crate::info::ConnectionExt;
use libc::{c_char, c_int};

/// # Safety
/// The caller must guarantee the pointer is valid.
#[no_mangle]
pub unsafe extern "C" fn airup_server_version(
    conn: &mut crate::blocking::Connection,
    buffer: *mut c_char,
    len: usize,
) -> c_int {
    match api_function_complex(|| Ok(conn.version()?)) {
        Some(s) => fill_c_string(&s, buffer, len),
        None => -1,
    }
}
