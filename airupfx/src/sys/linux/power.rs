//! Linux power management.

use crate::power::PowerManager;
use libc::{
    c_void, syscall, SYS_reboot, LINUX_REBOOT_CMD_HALT, LINUX_REBOOT_CMD_POWER_OFF,
    LINUX_REBOOT_CMD_RESTART, LINUX_REBOOT_MAGIC1, LINUX_REBOOT_MAGIC2,
};
use std::{convert::Infallible, ptr::NonNull};

#[derive(Default)]
pub struct Linux;
impl Linux {
    pub const GLOBAL: &'static Self = &Self;
}
impl PowerManager for Linux {
    fn poweroff(&self) -> std::io::Result<Infallible> {
        linux_reboot(LINUX_REBOOT_CMD_POWER_OFF, None)?;
        unreachable!()
    }

    fn reboot(&self) -> std::io::Result<Infallible> {
        linux_reboot(LINUX_REBOOT_CMD_RESTART, None)?;
        unreachable!()
    }

    fn halt(&self) -> std::io::Result<Infallible> {
        linux_reboot(LINUX_REBOOT_CMD_HALT, None)?;
        unreachable!()
    }
}

fn linux_reboot(cmd: libc::c_int, arg: Option<NonNull<c_void>>) -> std::io::Result<()> {
    let status = unsafe {
        syscall(
            SYS_reboot,
            LINUX_REBOOT_MAGIC1,
            LINUX_REBOOT_MAGIC2,
            cmd,
            arg,
        )
    };
    match status {
        0 => Ok(()),
        -1 => Err(std::io::Error::last_os_error()),
        _ => unreachable!(),
    }
}
