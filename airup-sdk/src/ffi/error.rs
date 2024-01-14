//! Error handling in C.

use libc::{c_char, c_void};
use std::cell::RefCell;

std::thread_local! {
    static LAST_ERROR: RefCell<Error> = RefCell::default();
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Error {
    pub code: u32,
    pub message: *mut c_char,
    pub payload: *mut c_void,
}
impl Error {
    pub fn new() -> Self {
        Self::default()
    }

    /// Delete the [`Error`] object.
    ///
    /// # Safety
    /// As [`Error`] implements [`Copy`], it's undefined behavior if this is called more than once on objects clone/copied from
    /// the same origin.
    pub unsafe fn delete(self) {
        if !self.message.is_null() {
            super::dealloc_c_string(self.message);
        }

        if !self.payload.is_null() {
            match self.code {
                16 => super::dealloc(self.payload as *mut i32),
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
            Some(x) => super::alloc(x),
            None => std::ptr::null_mut(),
        };

        Self {
            code: 16,
            message: super::alloc_c_string(&value.to_string()),
            payload,
        }
    }
}
impl From<crate::Error> for Error {
    fn from(value: crate::Error) -> Self {
        let payload = ApiError::from(value);
        let message = unsafe { super::duplicate_c_string(payload.message) };
        Self {
            code: 32,
            message,
            payload: super::alloc(payload),
        }
    }
}
impl Default for Error {
    fn default() -> Self {
        Self {
            code: 0,
            message: super::alloc_c_string("Undefined error ( 0 )"),
            payload: std::ptr::null_mut(),
        }
    }
}

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
            super::dealloc_c_string(p);
        }
    }
}
impl From<crate::Error> for ApiError {
    fn from(value: crate::Error) -> Self {
        let json = serde_json::to_value(&value)
            .expect("ApiError should always be able to serialize to JSON");
        let code = super::alloc_c_string(
            json.get("code")
                .expect("ApiError JSON should always contain `code` field")
                .as_str()
                .expect("`code` field of ApiError JSON should always be a string"),
        );
        let message = super::alloc_c_string(&value.to_string());
        let json = super::alloc_c_string(&json.to_string());

        Self {
            code,
            message,
            json,
        }
    }
}

#[no_mangle]
pub extern "C" fn airup_last_error() -> Error {
    LAST_ERROR.with(|x| *x.borrow())
}

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
