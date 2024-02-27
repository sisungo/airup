use super::error::{set_last_error, Error};
use libc::{c_char, c_int, c_void};

pub fn alloc_c_string(rs: &str) -> *mut c_char {
    std::ffi::CString::new(rs.replace('\0', "\u{FFFD}"))
        .expect("filtered string should never contain nul")
        .into_raw() as _
}

/// # Safety
/// The caller must guarantee the pointer is valid.
pub unsafe fn fill_c_string(rs: &str, cptr: *mut c_char, len: usize) -> c_int {
    if rs.len() > len - 1 {
        set_last_error(Error::buffer_too_small());
        return -1;
    }
    let rs = rs.replace('\0', "\u{FFFD}");
    for (n, byte) in rs.bytes().enumerate() {
        *cptr.add(n) = byte as _;
    }
    *cptr.add(rs.len()) = 0;
    0
}

/// # Safety
/// Calling this on the same pointer more than once may cause **double-free** problem.
pub unsafe fn dealloc_c_string(p: *mut c_char) {
    drop(std::ffi::CString::from_raw(p as *mut _));
}

/// # Safety
/// The caller must guarantee the pointer is valid.
pub unsafe fn duplicate_c_string(p: *const c_char) -> *mut c_char {
    std::ffi::CString::from(std::ffi::CStr::from_ptr(p)).into_raw() as _
}

pub fn alloc_voidptr<T>(value: T) -> *mut c_void {
    Box::into_raw(Box::new(value)) as _
}

/// # Safety
/// Calling this on the same pointer more than once may cause **double-free** problem.
pub unsafe fn dealloc<T>(ptr: *mut T) {
    drop(Box::from_raw(ptr));
}

pub fn api_function_complex<
    T,
    F: FnOnce() -> Result<Result<T, crate::Error>, Box<dyn std::error::Error>>,
>(
    f: F,
) -> Option<T> {
    match f() {
        Ok(Ok(x)) => Some(x),
        Ok(Err(err)) => {
            set_last_error(err.into());
            None
        }
        Err(err) => {
            let err = crate::Error::Io {
                message: err.to_string(),
            };
            set_last_error(err.into());
            None
        }
    }
}

pub fn api_function<F: FnOnce() -> Result<Result<(), crate::Error>, Box<dyn std::error::Error>>>(
    f: F,
) -> c_int {
    match api_function_complex(f) {
        Some(()) => 0,
        None => -1,
    }
}
