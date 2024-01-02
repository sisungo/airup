//! Error handling in C.

use libc::{c_char, c_void};

std::thread_local! {
    static LAST_ERROR: Error = Error::default();
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Error {
    pub code: u32,
    pub message: *const c_char,
    pub payload: *const c_void,
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
            super::dealloc_c_str(self.message);
        }

        if !self.payload.is_null() {
            unsafe {
                // NOTE: MEMORY LEAKING HERE! This may REFER TO A RUST TYPE which IMPLEMENTS DROP, however, current
                // implementation will NOT call [`Drop::drop`] as there is NO type notation. This must be fixed in future
                // releases.
                libc::free(self.payload as _);
            }
        }
    }
}
impl Default for Error {
    fn default() -> Self {
        Self {
            code: 0,
            message: super::alloc_c_str("Undefined error ( 0 )".into()),
            payload: std::ptr::null(),
        }
    }
}
