//! The Airupd application

use crate::*;
use airup_sdk::system::{QuerySystem, Status};
use airupfx::signal::*;
use std::{path::Path, sync::OnceLock};

static AIRUPD: OnceLock<Airupd> = OnceLock::new();

/// An instance of the Airupd app.
///
/// This is used in the singleton pattern, which means only one [`Airupd`] instance should exist in one `airupd` process.
#[derive(Debug)]
pub struct Airupd {
    /// The storage manager.
    pub storage: storage::Storage,

    /// The IPC context.
    pub ipc: ipc::Context,

    /// The lifetime manager of the `airupd` process.
    pub lifetime: lifetime::System,

    /// The milestone manager.
    pub milestones: milestones::Manager,

    /// The supervisor manager.
    pub supervisors: supervisor::Manager,

    /// The logger.
    pub logger: logger::Manager,

    /// Timestamp generated on creation of the struct.
    pub boot_timestamp: i64,

    /// Event bus.
    pub events: events::Bus,
}
impl Airupd {
    /// Queries information about the whole system.
    pub async fn query_system(&self) -> QuerySystem {
        let booted_since = self
            .query_milestone_stack()
            .last()
            .map(|x| x.finish_timestamp);

        QuerySystem {
            status: Status::Active,
            boot_timestamp: self.boot_timestamp,
            booted_since,
            is_booting: self.is_booting(),
            milestones: self.milestones.stack(),
            hostname: airupfx::env::host_name(),
            services: self.supervisors.list().await,
        }
    }

    /// Starts tasks to listen to UNIX signals.
    pub fn listen_signals(&'static self) {
        ignore_all([
            SIGHUP, SIGPIPE, SIGTTIN, SIGTTOU, SIGQUIT, SIGTERM, SIGUSR1, SIGUSR2,
        ]);
        signal(SIGINT, |_| async {
            self.enter_milestone("reboot".into()).await.ok();
        })
        .ok();
    }
}

/// Gets a reference to the unique, global [`Airupd`] instance.
///
/// # Panics
/// This method would panic if [`init`] was not previously called.
pub fn airupd() -> &'static Airupd {
    AIRUPD.get().unwrap()
}

/// Initializes the Airupd app for use of [`airupd`].
pub async fn init() {
    let object = Airupd {
        storage: storage::Storage::new().await,
        ipc: ipc::Context::new(),
        lifetime: lifetime::System::new(),
        milestones: milestones::Manager::new(),
        supervisors: supervisor::Manager::new(),
        logger: logger::Manager::new(),
        boot_timestamp: airupfx::time::timestamp_ms(),
        events: events::Bus::new(),
    };

    AIRUPD.set(object).unwrap();
}

pub async fn set_manifest_at(path: Option<&Path>) {
    if let Some(path) = path {
        std::env::set_var("AIRUP_OVERRIDE_MANIFEST_PATH", path);
        airup_sdk::build::set_manifest(
            serde_json::from_slice(
                &tokio::fs::read(path)
                    .await
                    .unwrap_log("failed to read overridden build manifest")
                    .await,
            )
            .unwrap_log("failed to parse overridden build manifest")
            .await,
        );
    }
}
