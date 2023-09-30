pub fn setsid() -> std::io::Result<libc::pid_t> {
    unsafe {
        let pgid = libc::setsid();
        match pgid {
            -1 => Err(std::io::Error::last_os_error()),
            x => Ok(x),
        }
    }
}

pub fn setgroups(groups: &[libc::gid_t]) -> std::io::Result<()> {
    unsafe {
        let pgid = libc::setgroups(groups.len() as _, groups.as_ptr()) as _;
        match pgid {
            0 => Ok(()),
            -1 => Err(std::io::Error::last_os_error()),
            _ => unreachable!(),
        }
    }
}