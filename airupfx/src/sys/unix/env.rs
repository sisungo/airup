//! Inspection and manipulation of the process's environment.

use std::{os::fd::AsRawFd, path::Path};

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

pub async fn setup_stdio<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let path = path.as_ref();

    loop {
        let file = tokio::fs::File::options()
            .read(true)
            .write(true)
            .open(path)
            .await?;
        if file.as_raw_fd() >= 3 {
            break Ok(());
        } else {
            std::mem::forget(file);
        }
    }
}

pub fn current_uid() -> sysinfo::Uid {
    sysinfo::Uid::try_from(unsafe { libc::getuid() as usize }).unwrap()
}
