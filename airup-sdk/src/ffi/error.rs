use std::cell::Cell;

std::thread_local! {
    static ERROR: Cell<Error> = const { Cell::new(Error::new()) };
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct Error {
    pub code: u32,
    pub payload: ErrorPayload,
}
impl Error {
    pub const AIRUP_ERROR_NOTHING: u32 = 0;
    pub const AIRUP_ERROR_IO: u32 = 32;

    pub const fn new() -> Self {
        Self {
            code: Self::AIRUP_ERROR_NOTHING,
            payload: ErrorPayload { nothing: () },
        }
    }
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self {
            code: Self::AIRUP_ERROR_IO,
            payload: ErrorPayload {
                sys_errno: value.raw_os_error().unwrap_or_default(),
            },
        }
    }
}
impl Default for Error {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union ErrorPayload {
    pub nothing: (),
    pub sys_errno: libc::c_int,
}

#[no_mangle]
pub extern "C" fn airup_get_error() -> Error {
    ERROR.get()
}

#[no_mangle]
pub extern "C" fn airup_set_error(error: Error) {
    ERROR.set(error);
}
