//! IPC primitives.

use crate::{error::AirupSDK_Error, ffi::*};
use libc::{c_char, c_int};
use std::{ffi::CStr, io::{Read, Write}, slice};

pub struct Connection(std::os::unix::net::UnixStream);
impl Connection {
    pub fn send(&mut self, bytes: &[u8]) -> std::io::Result<()> {
        self.0.write_all(&u64::to_le_bytes(bytes.len() as _))?;
        self.0.write_all(bytes)?;
        Ok(())
    }

    pub fn recv(&mut self) -> std::io::Result<Vec<u8>> {
        let mut len = [0u8; 8];
        self.0.read_exact(&mut len)?;
        let len = u64::from_le_bytes(len);
        let mut buf = vec![0u8; len as _];
        self.0.read_exact(&mut buf)?;
        Ok(buf)
    }
}

/// # Safety
/// This function is only safe is `path` is a valid NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn AirupSDK_OpenConnection(path: *const c_char) -> Option<Box<Connection>> {
    let path = CStr::from_ptr(path);
    let inner = std::os::unix::net::UnixStream::connect(&*path.to_string_lossy());
    let inner = match inner {
        Ok(x) => x,
        Err(e) => {
            crate::error::set(AirupSDK_Error::with_io(e));
            return None;
        }
    };
    Some(Box::new(Connection(inner)))
}

/// # Safety
/// This function is only safe is `data` is a valid array pointer and its length is `len`.
#[no_mangle]
pub unsafe extern "C" fn AirupSDK_SendMessage(
    conn: &mut Connection,
    data: *const u8,
    len: usize,
) -> c_int {
    if let Err(e) = conn.send(slice::from_raw_parts(data, len)) {
        crate::error::set(AirupSDK_Error::with_io(e));
        -1
    } else {
        0
    }
}

#[no_mangle]
pub extern "C" fn AirupSDK_RecvMessage(
    conn: &mut Connection,
    data: &mut *mut u8,
    len: &mut usize,
) -> c_int {
    match conn.recv() {
        Ok(x) => {
            let (ptr, _len) = allocate_bytes(&x);
            *data = ptr;
            *len = _len;
            0
        },
        Err(e) => {
            crate::error::set(AirupSDK_Error::with_io(e));
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn AirupSDK_CloseConnection(conn: Box<Connection>) {
    drop(conn);
}
