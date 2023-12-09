//! The `reboot` milestone preset series.

use ahash::AHashSet;
use airup_sdk::Error;

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
    Ok(())
}

/// Enters the `poweroff` milestone.
async fn enter_poweroff() -> Result<(), Error> {
    super::enter_milestone("poweroff".into(), &mut AHashSet::with_capacity(8))
        .await
        .ok();
    Ok(())
}

/// Enters the `halt` milestone.
async fn enter_halt() -> Result<(), Error> {
    super::enter_milestone("halt".into(), &mut AHashSet::with_capacity(8))
        .await
        .ok();
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
