use airup_sdk::system::ConnectionExt as _;
use clap::Parser;

/// Reboot, power-off or halt the system
#[derive(Debug, Clone, Parser)]
#[command(about)]
pub struct Cmdline {
    /// Specify the reboot mode
    #[arg(default_value = "reboot")]
    mode: String,
}

pub fn main(mut cmdline: Cmdline) -> anyhow::Result<()> {
    let mut conn = super::connect()?;

    if cmdline.mode == "userspace" {
        cmdline.mode = "userspace-reboot".into();
    }

    conn.enter_milestone(&cmdline.mode)?.ok();

    Ok(())
}
