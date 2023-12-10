//! FreeBSD power management.

use crate::power::PowerManager;
use libc::{RB_AUTOBOOT, RB_HALT, RB_POWEROFF};
use std::convert::Infallible;

#[derive(Default)]
pub struct FreeBsd;
impl FreeBsd {
    async fn prepare(&self) {
        crate::sys::process::kill_all(Duration::from_millis(5000)).await;
        crate::sys::fs::sync();
    }
}
#[async_trait::async_trait]
impl PowerManager for FreeBsd {
    async fn poweroff(&self) -> std::io::Result<Infallible> {
        self.prepare().await;
        freebsd_reboot(RB_POWEROFF)
    }

    async fn reboot(&self) -> std::io::Result<Infallible> {
        self.prepare().await;
        freebsd_reboot(RB_AUTOBOOT)
    }

    async fn halt(&self) -> std::io::Result<Infallible> {
        self.prepare().await;
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
