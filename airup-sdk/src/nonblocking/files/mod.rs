mod milestone;
mod system_conf;

pub use milestone::MilestoneExt;
pub use system_conf::SystemConfExt;

use crate::files::{Named, ReadError, Validate};
use serde::de::DeserializeOwned;
use std::path::PathBuf;

pub async fn read_merge<T: DeserializeOwned + Validate + Named>(
    paths: Vec<PathBuf>,
) -> Result<T, ReadError> {
    let Some(main_path) = paths.first() else {
        panic!("parameter `paths` must not be empty");
    };
    let main = tokio::fs::read_to_string(main_path).await?;
    let mut main = toml::from_str(&main)?;

    for path in &paths[1..] {
        let content = tokio::fs::read_to_string(path).await?;
        let patch = toml::from_str(&content)?;
        crate::files::merge(&mut main, &patch);
    }

    let mut object: T = T::deserialize(main)?;

    object.validate()?;
    object.set_name(main_path.file_stem().unwrap().to_string_lossy().into());

    Ok(object)
}
