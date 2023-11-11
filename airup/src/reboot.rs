use airup_sdk::prelude::*;
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
    #[arg(
        long,
        conflicts_with = "halt",
        conflicts_with = "poweroff",
        conflicts_with = "userspace"
    )]
    reboot: bool,

    /// Perform a userspace reboot
    #[arg(
        long,
        conflicts_with = "halt",
        conflicts_with = "poweroff",
        conflicts_with = "reboot"
    )]
    userspace: bool,

    args: Option<Vec<String>>,
}

/// Entrypoint of the `airup reboot` subprogram.
pub fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect()?;

    if cmdline.reboot {
        conn.reboot()??;
    }
    if cmdline.poweroff {
        conn.poweroff()??;
    }
    if cmdline.halt {
        conn.halt()??;
    }

    Ok(())
}
