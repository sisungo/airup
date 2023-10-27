//! Apple macOS power management.

#[link(name = "System")]
extern "C" {
    /// Reboots the system or halts the processor.
    ///
    /// This is a Apple Private API.
    fn reboot(howto: libc::c_int) -> libc::c_int;
}

use crate::power::PowerManager;
use std::convert::Infallible;

const RB_AUTOBOOT: libc::c_int = 0;
const RB_HALT: libc::c_int = 0x08;

#[derive(Default)]
pub struct MacOS;
impl PowerManager for MacOS {
    fn poweroff(&self) -> std::io::Result<Infallible> {
        macos_reboot(RB_HALT)
    }

    fn reboot(&self) -> std::io::Result<Infallible> {
        macos_reboot(RB_AUTOBOOT)
    }

    fn halt(&self) -> std::io::Result<Infallible> {
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
