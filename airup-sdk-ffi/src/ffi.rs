//! FFI helpers of the SDK.

use libc::c_char;
use std::os::raw::c_void;

pub fn allocate_cstr<S: AsRef<[u8]>>(s: S) -> *mut c_char {
    let mut s = s.as_ref().to_vec();
    s.iter_mut().filter(|x| **x == 0).for_each(|x| *x = b' ');
    s.push(0);
    unsafe {
        let ptr: *mut c_char = libc::malloc(s.len()) as _;
        libc::memcpy(ptr as _, s.as_ptr() as _, s.len());
        ptr
    }
}

/// # Safety
/// This function is only safe if `p` is returned by [`allocate_cstr`] and have never called [`deallocate_cstr`] before this
/// call.
pub unsafe fn deallocate_cstr(s: *mut c_char) {
    libc::free(s as _);
}

pub fn allocate_void<T>(value: T) -> *mut c_void {
    unsafe {
        let memory = libc::malloc(std::mem::size_of::<T>());
        std::ptr::write(memory as *mut T, value);
        memory
    }
}

/// # Safety
/// This function is only safe if `p` is returned by [`allocate_void`] and have never called [`deallocate_void`] before this
/// call.
pub unsafe fn deallocate_void(p: *mut c_void) {
    libc::free(p);
}

pub fn allocate_bytes(value: &[u8]) -> (*mut u8, usize) {
    unsafe {
        let memory = libc::malloc(value.len());
        libc::memcpy(memory, value.as_ptr() as _, value.len());
        (memory as _, value.len())
    }
}

/// # Safety
/// This function is only safe if `p` is returned by [`allocate_bytes`] and have never called [`deallocate_bytes`] before this
/// call.
pub unsafe fn deallocate_bytes(p: *mut u8) {
    libc::free(p as _);
}
