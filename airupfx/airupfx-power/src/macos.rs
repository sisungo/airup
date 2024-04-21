//! Apple macOS power management.

#[link(name = "System")]
extern "C" {
    /// Reboots the system or halts the processor.
    ///
    /// This is a Apple Private API.
    fn reboot(howto: libc::c_int) -> libc::c_int;
}

use crate::PowerManager;
use std::convert::Infallible;

const RB_AUTOBOOT: libc::c_int = 0;
const RB_HALT: libc::c_int = 0x08;

#[derive(Default)]
pub struct Power;
#[async_trait::async_trait]
impl PowerManager for Power {
    async fn poweroff(&self) -> std::io::Result<Infallible> {
        crate::unix::prepare().await;
        system_reboot(RB_HALT)
    }

    async fn reboot(&self) -> std::io::Result<Infallible> {
        crate::unix::prepare().await;
        system_reboot(RB_AUTOBOOT)
    }

    async fn halt(&self) -> std::io::Result<Infallible> {
        crate::unix::prepare().await;
        system_reboot(RB_HALT)
    }

    async fn userspace(&self) -> std::io::Result<Infallible> {
        crate::unix::prepare().await;
        airupfx_process::reload_image()
    }
}

fn system_reboot(cmd: libc::c_int) -> std::io::Result<Infallible> {
    let status = unsafe { reboot(cmd) };
    match status {
        -1 => Err(std::io::Error::last_os_error()),
        _ => unreachable!(),
    }
}

pub fn power_manager() -> &'static dyn PowerManager {
    &Power
}
