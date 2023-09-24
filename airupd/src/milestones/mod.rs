//! # Airup Milestones

mod early_boot;

use ahash::AHashSet;
use airup_sdk::Error;
use airupfx::{
    files::{
        milestone::{Item, Kind},
        Milestone,
    },
    prelude::*,
};

#[derive(Debug, Default)]
pub struct Manager {}
impl Manager {
    pub fn new() -> Self {
        Self::default()
    }
}

impl crate::app::Airupd {
    pub fn enter_milestone(&self, name: String) -> BoxFuture<Result<(), Error>> {
        if name == "early_boot" {
            Box::pin(early_boot::enter())
        } else {
            Box::pin(async { enter_milestone(self, name, &mut AHashSet::with_capacity(8)).await })
        }
    }
}

fn enter_milestone<'a>(
    airupd: &'a crate::app::Airupd,
    name: String,
    hist: &'a mut AHashSet<String>,
) -> BoxFuture<'a, Result<(), Error>> {
    Box::pin(async move {
        let name = name.strip_suffix(Milestone::SUFFIX).unwrap_or(&name);
        let def = match airupd.storage.milestones.get(name).await {
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

        for dep in def.manifest.milestone.dependencies.iter() {
            enter_milestone(airupd, dep.into(), hist).await.ok();
        }

        // By default, Airup sets `AIRUP_MILESTONE` environment variable to indicate services which milestone is the system
        // in as it is started.
        std::env::set_var("AIRUP_MILESTONE", name);

        // Starts services
        for item in def.services().await.iter() {
            exec_item(airupd, &def.manifest.milestone.kind, item).await;
        }

        Ok(())
    })
}

async fn exec_item(airupd: &crate::app::Airupd, kind: &Kind, item: &Item) {
    match item {
        Item::Cache(service) => {
            if let Err(err) = airupd.cache_service(service).await {
                tracing::error!(target: "console", "Failed to load \"{}\": {}", service, err);
            }
        }
        Item::Start(service) => match kind {
            Kind::Async => match airupd.start_service(service).await {
                Ok(_) | Err(Error::UnitStarted) => {
                    tracing::info!(target: "console", "Starting {}", display_name(airupd, service).await)
                }
                Err(err) => {
                    tracing::error!(target: "console", "Failed to start \"{}\": {}", service, err)
                }
            },
            Kind::Sync => {}
            Kind::Serial => match airupd.make_service_active(service).await {
                Ok(_) => {
                    tracing::info!(target: "console", "Starting {}", display_name(airupd, service).await)
                }
                Err(err) => {
                    tracing::error!(target: "console", "Failed to start {}: {}", display_name(airupd, service).await, err);
                }
            },
        },
    }
}

async fn display_name(airupd: &crate::app::Airupd, name: &str) -> String {
    airupd
        .query_service(name)
        .await
        .map(|x| x.service.display_name().into())
        .unwrap_or_else(|_| name.into())
}
