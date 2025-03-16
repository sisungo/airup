//! The `reboot` milestone preset series.

use crate::app::airupd;
use airup_sdk::Error;
use std::{collections::HashSet, time::Duration};
use tokio::task::{JoinHandle, JoinSet};

pub const PRESETS: &[&str] = &["reboot", "poweroff", "halt", "userspace-reboot"];

/// Enter a `reboot`-series milestone.
///
/// # Panics
/// This function would panic if `name` is not contained in [`PRESETS`].
pub async fn enter(name: &str) -> Result<(), Error> {
    _ = super::enter_milestone(name.into(), &mut HashSet::with_capacity(8)).await;
    let reboot_timeout = airupd().storage.config.system_conf.system.reboot_timeout;
    stop_all_services(Duration::from_millis(reboot_timeout as _)).await;

    match name {
        "reboot" => airupd().lifetime.reboot(),
        "poweroff" => airupd().lifetime.poweroff(),
        "halt" => airupd().lifetime.halt(),
        "userspace-reboot" => airupd().lifetime.userspace_reboot(),
        _ => unreachable!(),
    }

    Ok(())
}

/// Stops all running services.
async fn stop_all_services(timeout: Duration) {
    _ = tokio::time::timeout(timeout, async {
        let services = airupd().supervisors.list().await;
        let mut join_set = JoinSet::new();
        for service in services {
            join_set.spawn(stop_service_task(service));
        }
        join_set.join_all().await;
    })
    .await;
}

/// Spawns a task to interactively stop a service.
fn stop_service_task(service: String) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut error = None;
        match airupd().stop_service(&service).await {
            Ok(x) => {
                if let Err(err) = x.wait().await {
                    if !matches!(err, Error::NotStarted | Error::Unsupported { message: _ }) {
                        error = Some(err);
                    }
                } else {
                    tracing::info!(target: "console", "Stopping {}", super::display_name(&service).await);
                }
            }
            Err(err) => {
                if matches!(err, Error::NotFound | Error::NotStarted) {
                    return;
                }
                error = Some(err);
            }
        };
        if let Some(err) = error {
            tracing::error!(
                target: "console",
                "Failed to stop {}: {}",
                super::display_name(&service).await,
                err
            );
            _ = airupd().kill_service(&service).await;
        }
        _ = airupd().uncache_service(&service).await;
    })
}
