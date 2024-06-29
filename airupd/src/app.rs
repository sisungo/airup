//! The Airupd application

use crate::*;
use airup_sdk::system::{QuerySystem, Status};
use airupfx::signal::{
    self, SIGHUP, SIGINT, SIGPIPE, SIGQUIT, SIGTERM, SIGTTIN, SIGTTOU, SIGUSR1, SIGUSR2,
};
use std::{
    path::Path,
    sync::{
        atomic::{self, AtomicU32},
        OnceLock,
    },
};

static AIRUPD: OnceLock<Airupd> = OnceLock::new();

/// An instance of the Airupd app.
///
/// This is used in the singleton pattern, which means only one [`Airupd`] instance should exist in one `airupd` process.
#[derive(Debug)]
pub struct Airupd {
    /// The storage manager.
    pub storage: storage::Storage,

    /// The extension manager.
    pub extensions: extension::Extensions,

    /// The IPC context.
    pub ipc: rpc::Context,

    /// The lifetime manager of the `airupd` process.
    pub lifetime: lifetime::System,

    /// The milestone manager.
    pub milestones: milestones::Manager,

    /// The supervisor manager.
    pub supervisors: supervisor::Manager,

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

    /// Registers global signal hooks associated to this instance.
    pub fn set_signal_hooks(&'static self) {
        signal::init();

        signal::ignore_all([
            SIGPIPE, SIGTTIN, SIGTTOU, SIGQUIT, SIGTERM, SIGUSR1, SIGUSR2,
        ]);

        _ = signal::signal(SIGINT, |_| async {
            self.on_sigint().await;
        });

        _ = signal::signal(SIGHUP, |_| async {
            self.ipc.reload();
        });
    }

    /// Called when a `SIGINT` signal is received.
    async fn on_sigint(&self) {
        static COUNTER: AtomicU32 = AtomicU32::new(0);

        let counter = COUNTER.fetch_add(1, atomic::Ordering::SeqCst);

        if counter >= 8 {
            self.lifetime.reboot();
        } else if counter == 0 {
            _ = self.enter_milestone("reboot".into()).await;
        }
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
        extensions: extension::Extensions::new(),
        ipc: rpc::Context::new(),
        lifetime: lifetime::System::new(),
        milestones: milestones::Manager::new(),
        supervisors: supervisor::Manager::new(),
        boot_timestamp: airupfx::time::timestamp_ms(),
        events: events::Bus::new(),
    };

    AIRUPD.set(object).unwrap();
}

pub async fn set_manifest_at(path: Option<&Path>) {
    if let Some(path) = path {
        std::env::set_var("AIRUP_OVERRIDE_MANIFEST_PATH", path);
        airup_sdk::build::set_manifest(
            ciborium::from_reader(
                &tokio::fs::read(path)
                    .await
                    .unwrap_log("failed to read overridden build manifest")
                    .await[..],
            )
            .unwrap_log("failed to parse overridden build manifest")
            .await,
        );
    }
}
