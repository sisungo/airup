//! # Airup Milestones

pub mod early_boot;

use crate::app;
use ahash::AHashSet;
use airup_sdk::{
    files::{
        milestone::{Item, Kind},
        Milestone,
    },
    Error,
};
use airupfx::prelude::*;
use std::sync::{
    atomic::{self, AtomicBool},
    RwLock,
};

/// The milestone manager.
#[derive(Debug, Default)]
pub struct Manager {
    is_booting: AtomicBool,
    booted_since: RwLock<Option<i64>>,
}
impl Manager {
    /// Creates a new [`Manager`] instance.
    pub fn new() -> Self {
        Self::default()
    }
}

impl crate::app::Airupd {
    /// Enters the specific milestone.
    pub async fn enter_milestone(&self, name: String) -> Result<(), Error> {
        enter_milestone(name, &mut AHashSet::with_capacity(8)).await
    }

    /// Enters the specific milestone as bootstrap milestone.
    pub fn bootstrap_milestone(&'static self, name: String) {
        tokio::spawn(async move {
            self.milestones
                .is_booting
                .store(true, atomic::Ordering::Relaxed);

            self.enter_milestone(name).await.ok();

            *self.milestones.booted_since.write().unwrap() = Some(airupfx::time::timestamp_ms());
            self.milestones
                .is_booting
                .store(false, atomic::Ordering::Relaxed);
        });
    }

    /// Returns `true` if the system is booting.
    pub fn is_booting(&self) -> bool {
        self.milestones.is_booting.load(atomic::Ordering::Relaxed)
    }

    /// Returns a timestamp of boot completion.
    pub fn booted_since(&self) -> Option<i64> {
        self.milestones.booted_since.read().unwrap().clone()
    }
}

fn enter_milestone(name: String, hist: &mut AHashSet<String>) -> BoxFuture<'_, Result<(), Error>> {
    Box::pin(async move {
        let name = name.strip_suffix(Milestone::SUFFIX).unwrap_or(&name);
        let def = match app::airupd().storage.milestones.get(name).await {
            Ok(x) => x,
            Err(err) => {
                tracing::error!(target: "console", "Failed to enter milestone `{}`: {}", name, err);
                return Err(err.into());
            }
        };

        // Detects if dependency loop exists. If a dependency loop is detected, it's automatically broken, then a warning
        // event is recorded, and the method immediately returns.
        if !hist.insert(name.into()) {
            tracing::warn!(
                target: "console",
                "Dependency loop detected for milestone `{}`. Breaking loop.",
                def.display_name()
            );
            return Ok(());
        }

        tracing::info!(target: "console", "Entering milestone {}", def.display_name());

        // Enters dependency milestones
        for dep in def.manifest.milestone.dependencies.iter() {
            enter_milestone(dep.into(), hist).await.ok();
        }

        // By default, Airup sets `AIRUP_MILESTONE` environment variable to indicate services which milestone is the system
        // in as it is started.
        std::env::set_var("AIRUP_MILESTONE", name);

        // Starts services
        exec_milestone(&def).await;

        Ok(())
    })
}

async fn exec_milestone(def: &Milestone) {
    match def.manifest.milestone.kind {
        Kind::Async => exec_milestone_async(def).await,
        Kind::Serial => exec_milestone_serial(def).await,
        Kind::Sync => exec_milestone_sync(def).await,
    }
}

async fn exec_milestone_async(def: &Milestone) {
    let ace = once_cell::sync::Lazy::new(Ace::new);
    for item in def.items().await {
        match item {
            Item::Cache(service) => {
                if let Err(err) = app::airupd().cache_service(&service).await {
                    tracing::error!(target: "console", "Failed to load unit {}: {}", service, err);
                }
            }
            Item::Start(service) => match app::airupd().start_service(&service).await {
                Ok(_) => {
                    tracing::info!(target: "console", "Starting {}", display_name(&service).await)
                }
                Err(err) => {
                    tracing::error!(target: "console", "Failed to start {}: {}", service, err)
                }
            },
            Item::RunCmd(cmd) => {
                if let Err(err) = ace.run(&cmd).await {
                    tracing::error!(target: "console", "Failed to execute command `{cmd}`: {}", err);
                }
            }
        }
    }
}

async fn exec_milestone_serial(def: &Milestone) {
    let ace = once_cell::sync::Lazy::new(Ace::new);
    for item in def.items().await {
        match item {
            Item::Cache(service) => {
                if let Err(err) = app::airupd().cache_service(&service).await {
                    tracing::error!(target: "console", "Failed to load unit {}: {}", service, err);
                }
            }
            Item::Start(service) => match app::airupd().make_service_active(&service).await {
                Ok(_) => {
                    tracing::info!(target: "console", "Starting {}", display_name(&service).await)
                }
                Err(err) => {
                    tracing::error!(target: "console", "Failed to start {}: {}", display_name(&service).await, err);
                }
            },
            Item::RunCmd(cmd) => {
                if let Err(err) = run_wait(&ace, &cmd).await {
                    tracing::error!(target: "console", "Failed to execute command `{cmd}`: {}", err);
                }
            }
        }
    }
}

async fn exec_milestone_sync(def: &Milestone) {
    let ace = once_cell::sync::Lazy::new(Ace::new);
    let items = def.items().await;
    let mut commands = Vec::with_capacity(items.len());
    let mut handles = Vec::with_capacity(items.len());
    for item in items {
        match item {
            Item::Cache(service) => {
                if let Err(err) = app::airupd().cache_service(&service).await {
                    tracing::error!(target: "console", "Failed to load unit {}: {}", service, err);
                }
            }
            Item::Start(service) => match app::airupd().start_service(&service).await {
                Ok(x) => {
                    handles.push((service, x));
                }
                Err(err) => {
                    tracing::error!(target: "console", "Failed to start {}: {}", service, err);
                }
            },
            Item::RunCmd(cmd) => match ace.run(&cmd).await {
                Ok(x) => {
                    commands.push((cmd, x));
                }
                Err(err) => {
                    tracing::error!(target: "console", "Failed to execute command `{cmd}`: {}", err);
                }
            },
        }
    }

    for (name, handle) in handles {
        match handle.wait().await {
            Ok(_) | Err(Error::UnitStarted) => {
                tracing::info!(target: "console", "Starting {}", display_name(&name).await)
            }
            Err(err) => {
                tracing::error!(target: "console", "Failed to start {}: {}", display_name(&name).await, err);
            }
        }
    }

    for (cmd, child) in commands {
        if let Err(err) = child.wait().await {
            tracing::error!(target: "console", "Failed to execute command `{cmd}`: {}", err);
        }
    }
}

async fn display_name(name: &str) -> String {
    app::airupd()
        .query_service(name)
        .await
        .map(|x| x.service.display_name().into())
        .unwrap_or_else(|_| name.into())
}

pub async fn run_wait(ace: &Ace, cmd: &str) -> anyhow::Result<()> {
    ace.run_wait(cmd).await??;
    Ok(())
}
