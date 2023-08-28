//! # Milestones

#![allow(unstable_name_collisions)]

use super::ReadError;
use crate::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Represents to an Airup milestone.
#[derive(Debug, Clone)]
pub struct Milestone {
    pub name: String,
    pub manifest: Manifest,
    pub base_chain: DirChain,
}
impl Milestone {
    pub const EXTENSION: &str = "airm";
    pub const SUFFIX: &str = ".airm";

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
        let base_chain = DirChain::new(path);
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

    pub async fn services(&self) -> Vec<String> {
        let mut services = Vec::new();

        if let Ok(read_chain) = self
            .base_chain
            .read_chain()
            .await
            .inspect_err(|x| tracing::warn!("failed to read milestone directory: {x}"))
        {
            for i in read_chain {
                if i.to_string_lossy().ends_with(".list.airc") {
                    if let Some(path) = self.base_chain.find(&i).await.inspect_none(|| {
                        tracing::warn!("failed to get milestone chunkfile at `{:?}`", i)
                    }) {
                        tokio::fs::read_to_string(&path)
                            .await
                            .inspect_err(|e| {
                                tracing::warn!(
                                    "failed to read milestone chunkfile at `{:?}`: {}",
                                    i,
                                    e
                                )
                            })
                            .map(|x| x.lines().for_each(|y| services.push(String::from(y))))
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
    pub const FILE_NAME: &str = "milestone.airc";

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

    /// Latter services must be executed after the former service is active.
    Serial,
}
