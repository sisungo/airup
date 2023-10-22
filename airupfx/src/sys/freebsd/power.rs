//! FreeBSD power management.

use crate::power::PowerManager;
use libc::{RB_AUTOBOOT, RB_HALT, RB_POWEROFF};
use std::convert::Infallible;

#[derive(Default)]
pub struct FreeBsd;
impl PowerManager for FreeBsd {
    fn poweroff(&self) -> std::io::Result<Infallible> {
        freebsd_reboot(RB_POWEROFF)
    }

    fn reboot(&self) -> std::io::Result<Infallible> {
        freebsd_reboot(RB_AUTOBOOT)
    }

    fn halt(&self) -> std::io::Result<Infallible> {
        freebsd_reboot(RB_HALT)
    }
}

fn freebsd_reboot(cmd: libc::c_int) -> std::io::Result<Infallible> {
    let status = unsafe { reboot(cmd) };
    match status {
        -1 => Err(std::io::Error::last_os_error()),
        _ => unreachable!(),
    }
}
