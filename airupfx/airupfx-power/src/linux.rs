//! Linux power management.

use crate::PowerManager;
use libc::{
    c_void, syscall, SYS_reboot, LINUX_REBOOT_CMD_HALT, LINUX_REBOOT_CMD_POWER_OFF,
    LINUX_REBOOT_CMD_RESTART, LINUX_REBOOT_MAGIC1, LINUX_REBOOT_MAGIC2,
};
use std::{convert::Infallible, ptr::NonNull, time::Duration};

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
        reboot(LINUX_REBOOT_CMD_POWER_OFF, None)?;
        unreachable!()
    }

    async fn reboot(&self) -> std::io::Result<Infallible> {
        self.prepare().await;
        reboot(LINUX_REBOOT_CMD_RESTART, None)?;
        unreachable!()
    }

    async fn halt(&self) -> std::io::Result<Infallible> {
        self.prepare().await;
        reboot(LINUX_REBOOT_CMD_HALT, None)?;
        unreachable!()
    }
}

fn reboot(cmd: libc::c_int, arg: Option<NonNull<c_void>>) -> std::io::Result<()> {
    let status = unsafe {
        syscall(
            SYS_reboot,
            LINUX_REBOOT_MAGIC1,
            LINUX_REBOOT_MAGIC2,
            cmd,
            arg,
        )
    };
    if status < 0 {
        Err(std::io::ErrorKind::PermissionDenied.into())
    } else {
        Ok(())
    }
}

pub fn power_manager() -> &'static dyn PowerManager {
    &Power
}
