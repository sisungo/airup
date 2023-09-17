//! BSD-family (FreeBSD, NetBSD, DragonflyBSD, OpenBSD) power management.

use crate::power::PowerManager;
use libc::{RB_AUTOBOOT, RB_HALT, RB_POWEROFF};
use std::convert::Infallible;

#[derive(Default)]
pub struct Bsd;
impl Bsd {
    pub const GLOBAL: &'static Self = &Self;
}
impl PowerManager for Bsd {
    fn poweroff(&self) -> std::io::Result<Infallible> {
        bsd_reboot(RB_POWEROFF)
    }

    fn reboot(&self) -> std::io::Result<Infallible> {
        bsd_reboot(RB_AUTOBOOT)
    }

    fn halt(&self) -> std::io::Result<Infallible> {
        bsd_reboot(RB_HALT)
    }
}

fn bsd_reboot(cmd: libc::c_int) -> std::io::Result<Infallible> {
    let status = unsafe { reboot(cmd) };
    match status {
        -1 => Err(std::io::Error::last_os_error()),
        _ => unreachable!(),
    }
}
