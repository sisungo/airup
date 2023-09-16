//! The `early_boot` pseudo-milestone.

use airup_sdk::Error;
use airupfx::prelude::*;

/// Enters the `early_boot` pseudo-milestone.
pub async fn enter() -> Result<(), Error> {
    let ace = Ace::default();

    for &i in airupfx::config::build_manifest().early_cmds {
        if let Err(x) = run_wait(&ace, i).await {
            tracing::error!(target: "console", "Failed to execute command `{i}` in `early_boot` milestone: {}", x);
            airupfx::process::emergency();
        }
    }

    Ok(())
}

async fn run_wait(ace: &Ace, cmd: &str) -> anyhow::Result<()> {
    ace.run_wait(cmd).await??;
    Ok(())
}
