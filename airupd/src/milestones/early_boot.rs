//! The `early_boot` pseudo-milestone.

use airup_sdk::Error;
use airupfx::prelude::*;

/// Enters the `early_boot` pseudo-milestone.
pub async fn enter() -> Result<(), Error> {
    let ace = Ace::default();

    for &i in airupfx::config::build_manifest().early_cmds {
        if let Err(x) = ace.run(i).await {
            tracing::error!(target: "console", "Failed to execute command `{i}` in `early_boot` milestone: {}", x);
            airupfx::process::emergency();
        }
    }

    Ok(())
}
