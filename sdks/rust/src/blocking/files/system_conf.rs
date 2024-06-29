use crate::files::{ReadError, SystemConf};
use std::path::Path;

pub trait SystemConfExt {
    fn read_from<P: AsRef<Path>>(path: P) -> Result<SystemConf, ReadError>;
}
impl SystemConfExt for SystemConf {
    fn read_from<P: AsRef<Path>>(path: P) -> Result<Self, ReadError> {
        read_from(path.as_ref())
    }
}

fn read_from(path: &Path) -> Result<SystemConf, ReadError> {
    let s = std::fs::read_to_string(path)?;
    Ok(toml::from_str(&s)?)
}
