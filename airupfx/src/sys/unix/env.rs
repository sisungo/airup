use crate::process::Pid;

pub fn setsid() -> std::io::Result<Pid> {
    unsafe {
        let pgid = libc::setsid() as _;
        match pgid {
            -1 => Err(std::io::Error::last_os_error()),
            x => Ok(x),
        }
    }
}
