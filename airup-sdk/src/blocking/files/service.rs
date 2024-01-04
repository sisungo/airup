use crate::files::{ReadError, Service};
use std::path::PathBuf;

pub trait ServiceExt {
    /// Reads multiple [`Service`]'s from given paths, then merge them into a single [`Service`] instance. The first element in
    /// parameter `paths` is seen as the "main".
    ///
    /// # Panics
    /// Panics if parameter `paths` is empty.
    fn read_merge(paths: Vec<PathBuf>) -> Result<Service, ReadError>;
}
impl ServiceExt for Service {
    fn read_merge(paths: Vec<PathBuf>) -> Result<Service, ReadError> {
        let Some(main_path) = paths.first() else {
            panic!("parameter `paths` must not be empty");
        };
        let main = std::fs::read_to_string(main_path)?;
        let mut main: serde_json::Value = toml::from_str(&main)?;

        for path in &paths[1..] {
            let content = std::fs::read_to_string(path)?;
            let patch: serde_json::Value = toml::from_str(&content)?;
            json_patch::merge(&mut main, &patch);
        }

        let mut object: Self = serde_json::from_value(main)?;

        object.validate()?;
        object.name = main_path.file_stem().unwrap().to_string_lossy().into();

        Ok(object)
    }
}
