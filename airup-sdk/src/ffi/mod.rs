//! # Airup SDK for C
//! This module contains Airup SDK for the C programming language.

pub mod error;

pub fn alloc_c_string(rs: String) -> *const libc::c_char {
    std::ffi::CString::new(rs.replace('\0', "\u{FFFD}"))
        .expect("filtered string should never contain nul")
        .into_raw() as _
}

/// # Safety
/// Calling this on the same pointer more than once may cause **double-free** problem.
pub unsafe fn dealloc_c_string(p: *const libc::c_char) {
    drop(std::ffi::CString::from_raw(p as *mut _));
}
