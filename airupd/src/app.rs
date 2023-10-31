//! The Airupd application

use crate::{ipc, lifetime, milestones, storage::Storage, supervisor};
use airup_sdk::system::{QuerySystem, Status};
use airupfx::signal::*;
use std::sync::OnceLock;

static AIRUPD: OnceLock<Airupd> = OnceLock::new();

/// An instance of the Airupd app.
#[derive(Debug)]
pub struct Airupd {
    pub storage: Storage,
    pub ipc: ipc::Context,
    pub lifetime: lifetime::System,
    pub milestones: milestones::Manager,
    pub supervisors: supervisor::Manager,
    creation_time: i64,
}
impl Airupd {
    /// Initializes the Airupd app for use of [airupd].
    pub async fn init() {
        let object = Self {
            storage: Storage::new().await,
            ipc: ipc::Context::new(),
            lifetime: lifetime::System::new(),
            milestones: milestones::Manager::new(),
            supervisors: supervisor::Manager::new(),
            creation_time: airupfx::time::timestamp_ms(),
        };

        AIRUPD.set(object).unwrap();
    }

    /// Queries information about the whole system.
    pub async fn query_system(&self) -> QuerySystem {
        QuerySystem {
            status: Status::Active,
            status_since: self.creation_time,
            hostname: airupfx::env::host_name(),
            services: self.supervisors.list().await,
        }
    }

    /// Starts tasks to listen to UNIX signals.
    pub fn listen_signals(&'static self) {
        ignore_all([
            SIGHUP, SIGPIPE, SIGTTIN, SIGTTOU, SIGQUIT, SIGTERM, SIGUSR1, SIGUSR2,
        ]);
        signal(SIGINT, |_| async { self.lifetime.reboot() }).ok();
    }
}

/// Gets a reference to the unique, global [Airupd] instance.
pub fn airupd() -> &'static Airupd {
    AIRUPD.get().unwrap()
}
