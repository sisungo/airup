//! The `reboot` milestone preset series.

use crate::app::airupd;
use ahash::AHashSet;
use airup_sdk::Error;
use std::time::Duration;
use tokio::task::JoinHandle;

pub const PRESETS: &[&str] = &["reboot", "poweroff", "halt"];

/// Enter a `reboot`-series milestone.
///
/// # Panics
/// This function would panic if `name` is not contained in [`PRESETS`].
pub async fn enter(name: &str) -> Result<(), Error> {
    match name {
        "reboot" => enter_reboot().await,
        "poweroff" => enter_poweroff().await,
        "halt" => enter_halt().await,
        _ => panic!("Unexpected milestone `{name}`"),
    }
}

/// Enters the `reboot` milestone.
async fn enter_reboot() -> Result<(), Error> {
    super::enter_milestone("reboot".into(), &mut AHashSet::with_capacity(8))
        .await
        .ok();

    stop_all_services(Duration::from_millis(5000)).await;
    airupd().lifetime.reboot();

    Ok(())
}

/// Enters the `poweroff` milestone.
async fn enter_poweroff() -> Result<(), Error> {
    super::enter_milestone("poweroff".into(), &mut AHashSet::with_capacity(8))
        .await
        .ok();

    stop_all_services(Duration::from_millis(5000)).await;
    airupd().lifetime.poweroff();

    Ok(())
}

/// Enters the `halt` milestone.
async fn enter_halt() -> Result<(), Error> {
    super::enter_milestone("halt".into(), &mut AHashSet::with_capacity(8))
        .await
        .ok();

    stop_all_services(Duration::from_millis(5000)).await;
    airupd().lifetime.halt();

    Ok(())
}

/// Stops all running services.
async fn stop_all_services(timeout: Duration) {
    tokio::time::timeout(timeout, async {
        let services = airupd().supervisors.list().await;
        let mut join_handles = Vec::with_capacity(services.len());
        for service in services {
            join_handles.push(stop_service_task(service));
        }
        for join_handle in join_handles {
            join_handle.await.ok();
        }
    })
    .await
    .ok();
}

/// Spawns a task to interactively stop a service.
fn stop_service_task(service: String) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut error = None;
        match airupd().stop_service(&service).await {
            Ok(x) => {
                if let Err(err) = x.wait().await {
                    if !matches!(err, Error::UnitNotStarted) {
                        error = Some(err);
                    }
                } else {
                    tracing::info!(target: "console", "Stopping {}", super::display_name(&service).await);
                }
            }
            Err(err) => {
                if matches!(err, Error::UnitNotFound | Error::UnitNotStarted) {
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
        }
    })
}
