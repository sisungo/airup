use airup_sdk::system::ConnectionExt as _;
use clap::Parser;

/// Reboot, power-off or halt the system
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    /// Halt the device
    #[arg(
        long,
        conflicts_with = "poweroff",
        conflicts_with = "reboot",
        conflicts_with = "userspace"
    )]
    halt: bool,

    /// Power off the device
    #[arg(
        long,
        conflicts_with = "halt",
        conflicts_with = "reboot",
        conflicts_with = "userspace"
    )]
    poweroff: bool,

    /// Reboot the device
    #[arg(long, conflicts_with = "halt", conflicts_with = "poweroff")]
    reboot: bool,

    /// Perform a userspace reboot
    #[arg(long, conflicts_with = "halt", conflicts_with = "poweroff")]
    userspace: bool,
}

/// Entrypoint of the `airup reboot` subprogram.
pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect()?;

    if cmdline.reboot {
        conn.enter_milestone("reboot")?.ok();
    }
    if cmdline.poweroff {
        conn.enter_milestone("poweroff")?.ok();
    }
    if cmdline.halt {
        conn.enter_milestone("halt")?.ok();
    }
    if cmdline.userspace {
        conn.enter_milestone("userspace-reboot")?.ok();
    }

    Ok(())
}
