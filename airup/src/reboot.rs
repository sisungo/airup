use airup_sdk::prelude::*;
use airupfx::signal::SIGTERM;
use clap::Parser;

/// Reboots the system
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

    /// Don't kill processes
    #[arg(short, long)]
    no_kill: bool,

    /// Don't communicate with the init daemon
    no_ipc: bool,

    /// Don't commit filesystem caches to disk
    #[arg(long)]
    no_sync: bool,

    args: Option<Vec<String>>,
}

/// Entrypoint of the `airup reboot` subprogram.
pub async fn main(cmdline: Cmdline) -> anyhow::Result<()> {
    match cmdline.no_ipc {
        true => {
            if !cmdline.no_kill {
                airupfx::process::kill(-1, SIGTERM).await?;
            }
            if !cmdline.no_sync {
                airupfx::fs::sync();
            }

            if cmdline.reboot {
                airupfx::power::power_manager().reboot()?;
            }
            if cmdline.poweroff {
                airupfx::power::power_manager().poweroff()?;
            }
            if cmdline.halt {
                airupfx::power::power_manager().halt()?;
            }
        }
        false => {
            let mut conn = Connection::connect(airup_sdk::socket_path()).await?;
            if cmdline.reboot {
                conn.reboot().await??;
            }
            if cmdline.poweroff {
                conn.shutdown().await??;
            }
            if cmdline.halt {
                conn.halt().await??;
            }
        }
    };

    Ok(())
}
