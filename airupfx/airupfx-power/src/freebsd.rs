//! FreeBSD power management.

use crate::PowerManager;
use std::{convert::Infallible, time::Duration};

#[derive(Default)]
pub struct Power;
impl Power {
    async fn prepare(&self) {
        crate::unix::kill_all(Duration::from_millis(5000)).await;
        unsafe {
            libc::sync();
        }
    }
}
#[async_trait::async_trait]
impl PowerManager for Power {
    async fn poweroff(&self) -> std::io::Result<Infallible> {
        self.prepare().await;
        reboot(RB_POWEROFF)
    }

    async fn reboot(&self) -> std::io::Result<Infallible> {
        self.prepare().await;
        reboot(RB_AUTOBOOT)
    }

    async fn halt(&self) -> std::io::Result<Infallible> {
        self.prepare().await;
        reboot(RB_HALT)
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
