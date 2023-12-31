//! IPC primitives.

use crate::{error::AirupSDK_Error, ffi::*};
use libc::{c_char, c_int, c_void};
use serde::{Deserialize, Serialize};
use std::{
    ffi::CStr,
    io::{Read, Write},
    path::Path,
    slice,
    sync::OnceLock,
};

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
pub unsafe extern "C" fn AirupSDK_SendBytes(
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
pub extern "C" fn AirupSDK_RecvBytes(
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
        }
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

#[no_mangle]
pub extern "C" fn AirupSDK_PreferredIpcPath() -> *const c_char {
    static mut CACHED_DEFAULT: OnceLock<*const c_char> = OnceLock::new();

    unsafe {
        let airup_sock = libc::getenv("AIRUP_SOCK\0".as_ptr() as _);
        if !airup_sock.is_null() {
            return airup_sock as _;
        } else {
            *CACHED_DEFAULT.get_or_init(|| {
                let path = Path::new(crate::build::runtime_dir()).join("airupd.sock");
                allocate_cstr(&*path.to_string_lossy()) as _
            })
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub method: String,

    #[serde(alias = "param")]
    pub params: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "status", content = "payload")]
pub enum Response {
    Ok(serde_json::Value),
    Err(serde_json::Value),
}

#[no_mangle]
pub unsafe extern "C" fn AirupSDK_AllocateRequest(method: *const c_char) -> Box<Request> {
    Box::new(Request {
        method: CStr::from_ptr(method).to_string_lossy().into(),
        params: None,
    })
}

#[no_mangle]
pub extern "C" fn AirupSDK_DeallocateRequest(req: Box<Request>) {
    drop(req);
}

#[no_mangle]
pub unsafe extern "C" fn AirupSDK_PutRequestParameter(
    req: &mut Request,
    ty: c_int,
    val: *const c_void,
) {
    let value = match ty {
        1 => serde_json::Value::String(CStr::from_ptr(val as _).to_string_lossy().into()),
        _ => panic!("unknown ipc type"),
    };
    if req.params.is_none() {
        req.params = Some(value);
    } else if req.params.as_ref().unwrap().is_array() {
        req.params
            .as_mut()
            .unwrap()
            .as_array_mut()
            .unwrap()
            .push(value);
    } else {
        let origin = req.params.take().unwrap();
        let new = vec![origin, value];
        req.params = Some(new.into());
    }
}

#[no_mangle]
pub extern "C" fn AirupSDK_SerializeRequest(req: &Request) -> *const c_char {
    allocate_cstr(
        serde_json::to_string(req)
            .expect("a request object should never fail to be serialized into JSON"),
    )
}
