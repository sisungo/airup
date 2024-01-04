use crate::files::SystemConf;
use std::path::Path;

pub trait SystemConfExt {
    fn read_from<P: AsRef<Path>>(path: P) -> anyhow::Result<SystemConf>;
}
impl SystemConfExt for SystemConf {
    fn read_from<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        read_from(path.as_ref())
    }
}

fn read_from(path: &Path) -> anyhow::Result<SystemConf> {
    let s = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&s)?)
}
