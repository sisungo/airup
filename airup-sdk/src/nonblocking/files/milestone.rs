use crate::files::{milestone, Milestone, ReadError};
use crate::nonblocking::fs::DirChain;
use std::{future::Future, path::Path};

pub trait MilestoneExt {
    fn read_from<P: AsRef<Path>>(path: P) -> impl Future<Output = Result<Milestone, ReadError>>;
    fn items(&self) -> impl Future<Output = Vec<milestone::Item>>;
}
impl MilestoneExt for Milestone {
    async fn read_from<P: AsRef<Path>>(path: P) -> Result<Self, ReadError> {
        read_from(path.as_ref()).await
    }

    async fn items(&self) -> Vec<milestone::Item> {
        let mut services = Vec::new();
        let chain = DirChain::new(&self.base_dir);

        let Ok(read_chain) = chain.read_chain().await else {
            return services;
        };

        for i in read_chain {
            if !i.to_string_lossy().ends_with(".list.airf") {
                continue;
            }
            let Some(path) = chain.find(&i).await else {
                continue;
            };
            let Ok(list_str) = tokio::fs::read_to_string(&path).await else {
                continue;
            };
            for line in list_str.lines() {
                if let Ok(item) = line.parse() {
                    services.push(item);
                }
            }
        }

        services
    }
}

async fn read_from(path: &Path) -> Result<Milestone, ReadError> {
    let get_name = |p: &Path| -> Result<String, ReadError> {
        Ok(p.file_stem()
            .ok_or_else(|| ReadError::from("invalid milestone path"))?
            .to_string_lossy()
            .into())
    };
    let manifest = toml::from_str(
        &tokio::fs::read_to_string(path.join(milestone::Manifest::FILE_NAME)).await?,
    )?;
    let mut name = get_name(path)?;
    if name == "default" {
        name = get_name(&tokio::fs::canonicalize(path).await?)?;
    }

    Ok(Milestone {
        name,
        manifest,
        base_dir: path.into(),
    })
}
