use crate::files::{ReadError, SystemConf};
use std::{future::Future, path::Path};

pub trait SystemConfExt {
    fn read_from<P: AsRef<Path>>(path: P) -> impl Future<Output = Result<SystemConf, ReadError>>;
}
impl SystemConfExt for SystemConf {
    async fn read_from<P: AsRef<Path>>(path: P) -> Result<Self, ReadError> {
        read_from(path.as_ref()).await
    }
}

async fn read_from(path: &Path) -> Result<SystemConf, ReadError> {
    let s = tokio::fs::read_to_string(path).await?;
    Ok(toml::from_str(&s)?)
}
