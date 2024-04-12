//! # Airup Milestones

pub mod early_boot;
mod reboot;

use crate::app::{self, airupd};
use airup_sdk::{
    files::{
        milestone::{Item, Kind},
        Milestone,
    },
    prelude::*,
    system::EnteredMilestone,
    Error,
};
use airupfx::prelude::*;
use std::{
    collections::HashSet,
    sync::{
        atomic::{self, AtomicBool},
        RwLock,
    },
};

/// The milestone manager.
#[derive(Debug, Default)]
pub struct Manager {
    is_booting: AtomicBool,
    stack: RwLock<Vec<EnteredMilestone>>,
}
impl Manager {
    /// Creates a new [`Manager`] instance.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn stack(&self) -> Vec<EnteredMilestone> {
        self.stack.read().unwrap().clone()
    }
}

impl crate::app::Airupd {
    /// Enters the specific milestone.
    pub async fn enter_milestone(&self, name: String) -> Result<(), Error> {
        let name = name.strip_suffix(".airm").unwrap_or(&name).to_owned();
        if reboot::PRESETS.contains(&&name[..]) {
            reboot::enter(&name).await
        } else {
            enter_milestone(name, &mut HashSet::with_capacity(8)).await
        }
    }

    /// Enters the specific milestone as bootstrap milestone.
    pub fn bootstrap_milestone(&'static self, name: String) {
        tokio::spawn(async move {
            self.milestones
                .is_booting
                .store(true, atomic::Ordering::Relaxed);

            self.enter_milestone(name).await.ok();

            self.milestones
                .is_booting
                .store(false, atomic::Ordering::Relaxed);
        });
    }

    /// Returns `true` if the system is booting.
    pub fn is_booting(&self) -> bool {
        self.milestones.is_booting.load(atomic::Ordering::Relaxed)
    }

    /// Queries the milestone stack.
    pub fn query_milestone_stack(&self) -> Vec<EnteredMilestone> {
        self.milestones.stack.read().unwrap().clone()
    }
}

async fn enter_milestone(name: String, hist: &mut HashSet<String>) -> Result<(), Error> {
    let def = match app::airupd().storage.milestones.get(&name).await {
        Ok(x) => x,
        Err(err) => {
            tracing::error!(target: "console", "Failed to enter milestone `{}`: {}", name, err);
            return Err(err.into());
        }
    };

    // Detects if dependency ring exists. If a dependency ring is detected, it's automatically broken, then a warning
    // event is recorded, and the method immediately returns.
    if !hist.insert(name.clone()) {
        tracing::warn!(
            target: "console",
            "Dependency loop detected for milestone `{}`. Breaking loop.",
            def.display_name()
        );
        return Ok(());
    }

    tracing::info!(target: "console", "Entering milestone {}", def.display_name());
    let begin_timestamp = airupfx::time::timestamp_ms();

    // Enters dependency milestones
    for dep in def.manifest.milestone.dependencies.iter() {
        Box::pin(enter_milestone(dep.into(), hist)).await.ok();
    }

    // By default, Airup sets `AIRUP_MILESTONE` environment variable to indicate services which milestone is the system
    // in as it is started.
    std::env::set_var("AIRUP_MILESTONE", &name);

    // Starts services
    exec_milestone(&def).await;

    // Record the milestone as entered
    let finish_timestamp = airupfx::time::timestamp_ms();
    let record = EnteredMilestone {
        name: name.clone(),
        begin_timestamp,
        finish_timestamp,
    };
    airupd().milestones.stack.write().unwrap().push(record);

    Ok(())
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
                    tracing::error!(target: "console", "Failed to load service {}: {}", service, err);
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
            Item::Run(cmd) => {
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
                    tracing::error!(target: "console", "Failed to load service {}: {}", service, err);
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
            Item::Run(cmd) => {
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
                    tracing::error!(target: "console", "Failed to load service {}: {}", service, err);
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
            Item::Run(cmd) => match ace.run(&cmd).await {
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
            Ok(_) | Err(Error::Started) => {
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
        .map(|x| x.definition.display_name().into())
        .unwrap_or_else(|_| format!("`{name}`"))
}

pub async fn run_wait(ace: &Ace, cmd: &str) -> anyhow::Result<()> {
    ace.run_wait(cmd).await??;
    Ok(())
}
