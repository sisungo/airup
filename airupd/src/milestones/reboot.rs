//! The `reboot` milestone preset series.

use crate::app::airupd;
use ahash::AHashSet;
use airup_sdk::Error;
use tokio::task::JoinHandle;
use std::time::Duration;

pub const PRESETS: &[&str] = &["reboot", "poweroff", "halt", "ctrlaltdel"];

/// Enter a `reboot`-series milestone.
///
/// # Panics
/// This function would panic if `name` is not contained in [`PRESETS`].
pub async fn enter(name: &str) -> Result<(), Error> {
    match name {
        "reboot" => enter_reboot().await,
        "poweroff" => enter_poweroff().await,
        "halt" => enter_halt().await,
        "ctrlaltdel" => enter_ctrlaltdel().await,
        _ => panic!("Unexpected milestone `{name}`"),
    }
}

/// Enters the `reboot` milestone.
async fn enter_reboot() -> Result<(), Error> {
    super::enter_milestone("reboot".into(), &mut AHashSet::with_capacity(8))
        .await
        .ok();

    stop_all_services(Duration::from_millis(5000)).await;
    
    Ok(())
}

/// Enters the `poweroff` milestone.
async fn enter_poweroff() -> Result<(), Error> {
    super::enter_milestone("poweroff".into(), &mut AHashSet::with_capacity(8))
        .await
        .ok();

    stop_all_services(Duration::from_millis(5000)).await;

    Ok(())
}

/// Enters the `halt` milestone.
async fn enter_halt() -> Result<(), Error> {
    super::enter_milestone("halt".into(), &mut AHashSet::with_capacity(8))
        .await
        .ok();

    stop_all_services(Duration::from_millis(5000)).await;

    Ok(())
}

/// Enters the `ctrlaltdel` milestone.
///
/// The `ctrlaltdel` milestone issues system reboot by default. If `ctrlaltdel.airm` is defined in `$milestone_dir`, the
/// default behavior is overrided.
async fn enter_ctrlaltdel() -> Result<(), Error> {
    let local = super::enter_milestone("ctrlaltdel".into(), &mut AHashSet::with_capacity(8)).await;

    match local {
        Ok(()) => Ok(()),
        Err(_) => enter_reboot().await,
    }
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
        match airupd().stop_service(&service).await {
            Ok(_) => {
                tracing::info!("Stopping {}", super::display_name(&service).await);
            }
            Err(err) => {
                if matches!(err, Error::UnitNotFound | Error::UnitNotStarted) {
                    return;
                }
                tracing::error!(
                    "Failed to stop {}: {}",
                    super::display_name(&service).await,
                    err
                );
            }
        };
    })
}