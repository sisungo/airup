use crate::files::SystemConf;
use std::{future::Future, path::Path};

pub trait SystemConfExt {
    fn read_from<P: AsRef<Path>>(path: P) -> impl Future<Output = anyhow::Result<SystemConf>>;
}
impl SystemConfExt for SystemConf {
    async fn read_from<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        read_from(path.as_ref()).await
    }
}

async fn read_from(path: &Path) -> anyhow::Result<SystemConf> {
    let s = tokio::fs::read_to_string(path).await?;
    Ok(toml::from_str(&s)?)
}
