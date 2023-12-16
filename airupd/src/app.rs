//! The Airupd application

use crate::{ipc, lifetime, logger, milestones, storage::Storage, supervisor};
use airup_sdk::system::{QuerySystem, Status};
use airupfx::signal::*;
use std::sync::OnceLock;

static AIRUPD: OnceLock<Airupd> = OnceLock::new();

/// An instance of the Airupd app.
///
/// This is used in the singleton pattern, which means only one [`Airupd`] instance should exist in one `airupd` process.
#[derive(Debug)]
pub struct Airupd {
    /// The storage manager.
    pub storage: Storage,

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
}
impl Airupd {
    /// Initializes the Airupd app for use of [`airupd`].
    pub async fn init() {
        let object = Self {
            storage: Storage::new().await,
            ipc: ipc::Context::new(),
            lifetime: lifetime::System::new(),
            milestones: milestones::Manager::new(),
            supervisors: supervisor::Manager::new(),
            logger: logger::Manager::new(),
            boot_timestamp: airupfx::time::timestamp_ms(),
        };

        AIRUPD.set(object).unwrap();
    }

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
            self.enter_milestone("ctrlaltdel".into()).await.ok();
        })
        .ok();
    }
}

/// Gets a reference to the unique, global [`Airupd`] instance.
///
/// # Panics
/// This method would panic if `Airupd::init` was not previously called.
pub fn airupd() -> &'static Airupd {
    AIRUPD.get().unwrap()
}
