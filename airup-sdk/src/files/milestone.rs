//! # Milestones

#![allow(unstable_name_collisions)]

use super::ReadError;
use airupfx::prelude::*;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use std::{path::Path, str::FromStr};

/// Represents to an Airup milestone.
#[derive(Debug, Clone)]
pub struct Milestone {
    pub name: String,
    pub manifest: Manifest,
    pub base_chain: DirChain<'static>,
}
impl Milestone {
    pub const EXTENSION: &'static str = "airm";
    pub const SUFFIX: &'static str = ".airm";

    /// Reads a [Milestone] from given directory.
    pub async fn read_from<P: AsRef<Path>>(path: P) -> Result<Self, ReadError> {
        Self::_read_from(path.as_ref()).await
    }

    /// Returns the name to display for this service.
    pub fn display_name(&self) -> &str {
        self.manifest
            .milestone
            .display_name
            .as_deref()
            .unwrap_or(&self.name)
    }

    async fn _read_from(path: &Path) -> Result<Self, ReadError> {
        let get_name = |p: &Path| -> Result<String, ReadError> {
            Ok(p.file_stem()
                .ok_or_else(|| ReadError::from("invalid milestone path"))?
                .to_string_lossy()
                .into())
        };
        let base_chain = DirChain::new(path.to_owned());
        let manifest = Manifest::read_from(path.join(Manifest::FILE_NAME)).await?;
        let mut name = get_name(path)?;
        if name == "default" {
            name = get_name(&tokio::fs::canonicalize(path).await?)?;
        }

        Ok(Self {
            name,
            manifest,
            base_chain,
        })
    }

    pub async fn items(&self) -> Vec<Item> {
        let mut services = Vec::new();

        if let Ok(read_chain) = self.base_chain.read_chain().await {
            for i in read_chain {
                if i.to_string_lossy().ends_with(".list.airc") {
                    if let Some(path) = self.base_chain.find(&i).await {
                        tokio::fs::read_to_string(&path)
                            .await
                            .map(|x| {
                                x.lines().for_each(|y| {
                                    if let Ok(item) = y.parse() {
                                        services.push(item);
                                    }
                                })
                            })
                            .ok();
                    }
                }
            }
        }

        services
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub milestone: Metadata,
}
impl Manifest {
    pub const FILE_NAME: &'static str = "milestone.airc";

    async fn read_from<P: AsRef<Path>>(path: P) -> Result<Self, ReadError> {
        Ok(toml::from_str(&tokio::fs::read_to_string(path).await?)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    RunCmd(String),
}
impl FromStr for Item {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splited = s.splitn(2, ' ');
        let verb = splited
            .next()
            .ok_or_else(|| anyhow!("missing verb for milestone items"))?;
        let entity = splited
            .next()
            .ok_or_else(|| anyhow!("missing entity for milestone items"))?;
        match verb {
            "cache" => Ok(Self::Cache(entity.into())),
            "start" => Ok(Self::Start(entity.into())),
            "run_cmd" => Ok(Self::RunCmd(entity.into())),
            _ => Err(anyhow!(
                "verb `{verb}` is not considered for milestone items"
            )),
        }
    }
}
