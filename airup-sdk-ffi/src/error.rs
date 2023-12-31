//! Error handling module.

#![allow(nonstandard_style)]

use crate::ffi::*;
use libc::{c_char, c_void};
use std::cell::RefCell;

std::thread_local! {
    static LAST_ERROR: RefCell<AirupSDK_Error> = RefCell::default();
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct AirupSDK_Error {
    pub code: u32,
    pub message: *mut c_char,
    pub info: *mut c_void,
}
impl AirupSDK_Error {
    /// Drops all allocated system resource (e.g. heap memory) referred by this structure.
    ///
    /// # Safety
    /// This function invalidates all [`AirupSDK_Error`]'s with the same system resources.
    pub unsafe fn delete(self) {
        deallocate_cstr(self.message);
        if !self.info.is_null() {
            deallocate_void(self.info);
        }
    }

    pub fn with_io(e: std::io::Error) -> Self {
        Self {
            code: 1,
            message: allocate_cstr(e.to_string()),
            info: allocate_void(e.raw_os_error().unwrap_or(0)),
        }
    }
}
impl Default for AirupSDK_Error {
    fn default() -> Self {
        Self {
            code: 0,
            message: allocate_cstr("Undefined error ( 0 )"),
            info: std::ptr::null_mut(),
        }
    }
}

#[no_mangle]
pub extern "C" fn AirupSDK_GetLastError() -> AirupSDK_Error {
    LAST_ERROR.with(|x| *x.borrow())
}

pub fn set(new: AirupSDK_Error) {
    LAST_ERROR.with(|x| {
        let mut x = x.borrow_mut();
        unsafe {
            x.delete();
        }
        *x = new;
    })
}
