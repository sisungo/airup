//! FreeBSD power management.

use crate::PowerManager;
use std::{convert::Infallible, time::Duration};

#[derive(Default)]
pub struct Power;
#[async_trait::async_trait]
impl PowerManager for Power {
    async fn poweroff(&self) -> std::io::Result<Infallible> {
        crate::unix::prepare().await;
        reboot(RB_POWEROFF)
    }

    async fn reboot(&self) -> std::io::Result<Infallible> {
        crate::unix::prepare().await;
        reboot(RB_AUTOBOOT)
    }

    async fn halt(&self) -> std::io::Result<Infallible> {
        crate::unix::prepare().await;
        reboot(RB_HALT)
    }

    async fn userspace(&self) -> std::io::Result<Infallible> {
        crate::unix::prepare().await;
        airupfx_process::reload_image()
    }
}

fn reboot(cmd: libc::c_int) -> std::io::Result<Infallible> {
    let status = unsafe { libc::reboot(cmd) };
    match status {
        -1 => Err(std::io::Error::last_os_error()),
        _ => unreachable!(),
    }
}

pub fn power_manager() -> &'static dyn PowerManager {
    &Power
}
