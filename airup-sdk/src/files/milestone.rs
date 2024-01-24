//! # Milestones

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

/// Represents to an Airup milestone.
#[derive(Debug, Clone)]
pub struct Milestone {
    pub name: String,
    pub manifest: Manifest,
    pub base_dir: PathBuf,
}
impl Milestone {
    /// Returns the name to display for this service.
    pub fn display_name(&self) -> &str {
        self.manifest
            .milestone
            .display_name
            .as_deref()
            .unwrap_or(&self.name)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub milestone: Metadata,
}
impl Manifest {
    pub const FILE_NAME: &'static str = "milestone.airf";
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Metadata {
    /// Display of the milestone.
    pub display_name: Option<String>,

    /// Description of the milestone.
    pub description: Option<String>,

    /// Dependencies of the milestone.
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// Tags of the milestone.
    #[serde(default)]
    pub tags: Vec<String>,

    #[serde(default)]
    pub kind: Kind,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// Services are asynchronously started.
    #[default]
    Async,

    /// Services are asynchronously started, but the milestone is completed when they are active.
    Sync,

    /// Latter services must be executed after the former service is active.
    Serial,
}

#[derive(Debug, Clone)]
pub enum Item {
    Cache(String),
    Start(String),
    Run(String),
}
impl FromStr for Item {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splited = s.splitn(2, ' ');
        let verb = splited
            .next()
            .ok_or_else(|| anyhow!("missing verb in milestone items"))?;
        let entity = splited
            .next()
            .ok_or_else(|| anyhow!("missing unit in milestone items"))?;
        match verb {
            "cache" => Ok(Self::Cache(entity.into())),
            "start" => Ok(Self::Start(entity.into())),
            "run" => Ok(Self::Run(entity.into())),
            _ => Err(anyhow!(
                "verb `{verb}` is not considered in milestone items"
            )),
        }
    }
}
