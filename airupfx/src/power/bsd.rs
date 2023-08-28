//! BSD-family (FreeBSD, NetBSD, DragonflyBSD, OpenBSD) power management.

use super::PowerManager;
use libc::{RB_AUTOBOOT, RB_HALT, RB_POWEROFF};
use std::{convert::Infallible, ptr::NonNull};

#[derive(Default)]
pub struct Linux;
impl PowerManager for Linux {
    fn shutdown(&self) -> std::io::Result<Infallible> {
        bsd_reboot(RB_POWEROFF)
    }

    fn reboot(&self) -> std::io::Result<Infallible> {
        bsd_reboot(RB_AUTOBOOT, None)
    }

    fn halt(&self) -> std::io::Result<Infallible> {
        bsd_reboot(RB_HALT, None)
    }
}

fn bsd_reboot(cmd: libc::c_int) -> std::io::Result<Infallible> {
    let status = unsafe { reboot(cmd) };
    match status {
        -1 => Err(std::io::Error::last_os_error()),
        _ => unreachable!(),
    }
}
