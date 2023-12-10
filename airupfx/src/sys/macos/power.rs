//! Apple macOS power management.

#[link(name = "System")]
extern "C" {
    /// Reboots the system or halts the processor.
    ///
    /// This is a Apple Private API.
    fn reboot(howto: libc::c_int) -> libc::c_int;
}

use crate::power::PowerManager;
use std::{convert::Infallible, time::Duration};

const RB_AUTOBOOT: libc::c_int = 0;
const RB_HALT: libc::c_int = 0x08;

#[derive(Default)]
pub struct MacOS;
impl MacOS {
    async fn prepare(&self) {
        crate::sys::process::kill_all(Duration::from_millis(5000)).await;
        crate::sys::fs::sync();
    }
}
#[async_trait::async_trait]
impl PowerManager for MacOS {
    async fn poweroff(&self) -> std::io::Result<Infallible> {
        self.prepare().await;
        macos_reboot(RB_HALT)
    }

    async fn reboot(&self) -> std::io::Result<Infallible> {
        self.prepare().await;
        macos_reboot(RB_AUTOBOOT)
    }

    async fn halt(&self) -> std::io::Result<Infallible> {
        self.prepare().await;
        macos_reboot(RB_HALT)
    }
}

fn macos_reboot(cmd: libc::c_int) -> std::io::Result<Infallible> {
    let status = unsafe { reboot(cmd) };
    match status {
        -1 => Err(std::io::Error::last_os_error()),
        _ => unreachable!(),
    }
}
