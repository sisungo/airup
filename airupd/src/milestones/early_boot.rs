//! The `early_boot` pseudo-milestone.

use airupfx::prelude::*;

/// Enters the `early_boot` pseudo-milestone.
pub async fn enter() {
    let ace = Ace::default();

    for i in &airupfx::config::build_manifest().early_cmds {
        if let Err(x) = super::run_wait(&ace, i).await {
            tracing::error!(target: "console", "Failed to execute command `{i}` in `early_boot` milestone: {}", x);
            airupfx::process::emergency();
        }
    }
}
