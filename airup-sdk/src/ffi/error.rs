//! Error handling in C.

use super::util::*;
use libc::{c_char, c_void};
use std::cell::RefCell;

std::thread_local! {
    static LAST_ERROR: RefCell<Error> = RefCell::default();
}

#[no_mangle]
pub extern "C" fn airup_last_error() -> Error {
    LAST_ERROR.with(|x| *x.borrow())
}

/// Represents to an error from the Airup SDK for C.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Error {
    pub code: u32,
    pub message: *mut c_char,
    pub payload: *mut c_void,
}
impl Error {
    /// Creates a default [`Error`] instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an [`Error`] instance with the `AIRUP_EBUFTOOSMALL` error code.
    pub fn buffer_too_small() -> Self {
        Self {
            code: 64,
            message: alloc_c_string("buffer too small"),
            payload: std::ptr::null_mut(),
        }
    }

    /// Delete the [`Error`] object.
    ///
    /// # Safety
    /// As [`Error`] implements [`Copy`], it's undefined behavior if this is called more than once on objects clone/copied from
    /// the same origin.
    pub unsafe fn delete(self) {
        if !self.message.is_null() {
            dealloc_c_string(self.message);
        }

        if !self.payload.is_null() {
            match self.code {
                16 => dealloc(self.payload as *mut i32),
                32 => {
                    let val = Box::from_raw(self.payload as *mut ApiError);
                    val.delete();
                }
                _ => (), // unknown code
            }
        }
    }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        let payload = match value.raw_os_error() {
            Some(x) => alloc_voidptr(x),
            None => std::ptr::null_mut(),
        };

        Self {
            code: 16,
            message: alloc_c_string(&value.to_string()),
            payload,
        }
    }
}
impl From<crate::Error> for Error {
    fn from(value: crate::Error) -> Self {
        let payload = ApiError::from(value);
        let message = unsafe { duplicate_c_string(payload.message) };
        Self {
            code: 32,
            message,
            payload: alloc_voidptr(payload),
        }
    }
}
impl Default for Error {
    fn default() -> Self {
        Self {
            code: 0,
            message: alloc_c_string("Undefined error ( 0 )"),
            payload: std::ptr::null_mut(),
        }
    }
}

/// C-friendly representation of [`crate::error::ApiError`]. This is the payload type of the error code `AIRUP_EAPI`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ApiError {
    pub code: *mut c_char,
    pub message: *mut c_char,
    pub json: *mut c_char,
}
impl ApiError {
    /// Delete the [`ApiError`] object.
    ///
    /// # Safety
    /// As [`ApiError`] implements [`Copy`], it's undefined behavior if this is called more than once on objects clone/copied
    /// from the same origin.
    pub unsafe fn delete(self) {
        for p in [self.code, self.message, self.json] {
            dealloc_c_string(p);
        }
    }
}
impl From<crate::Error> for ApiError {
    fn from(value: crate::Error) -> Self {
        let json = serde_json::to_value(&value)
            .expect("ApiError should always be able to serialize to JSON");
        let code = alloc_c_string(
            json.get("code")
                .expect("ApiError JSON should always contain `code` field")
                .as_str()
                .expect("`code` field of ApiError JSON should always be a string"),
        );
        let message = alloc_c_string(&value.to_string());
        let json = alloc_c_string(&json.to_string());

        Self {
            code,
            message,
            json,
        }
    }
}

/// Sets current thread's Airup error.
pub fn set_last_error(new: Error) {
    LAST_ERROR.with(|x| {
        let mut re = x.borrow_mut();

        // SAFETY: It is UB if pointers in previous [`Error`] returned by [`airup_last_error`] are accessed. However, they are
        // only accessible by using `unsafe`.
        unsafe {
            re.delete();
        }

        *re = new;
    });
}
