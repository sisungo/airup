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
    let mut main: serde_json::Value = toml::from_str(&main)?;

    for path in &paths[1..] {
        let content = tokio::fs::read_to_string(path).await?;
        let patch: serde_json::Value = toml::from_str(&content)?;
        json_patch::merge(&mut main, &patch);
    }

    let mut object: T = serde_json::from_value(main)?;

    object.validate()?;
    object.set_name(main_path.file_stem().unwrap().to_string_lossy().into());

    Ok(object)
}
