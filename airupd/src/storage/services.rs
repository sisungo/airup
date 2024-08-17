//! Represents to Airup's services directory.

use airup_sdk::{
    files::{ReadError, Service},
    nonblocking::files,
    prelude::*,
};
use std::path::PathBuf;

/// Represents to Airup's services directory.
#[derive(Debug)]
pub struct Services {
    base_chain: DirChain<'static>,
}
impl From<DirChain<'static>> for Services {
    fn from(val: DirChain<'static>) -> Self {
        Self { base_chain: val }
    }
}
impl Services {
    /// Creates a new [`Services`] instance.
    pub fn new() -> Self {
        Self {
            base_chain: DirChain::new(airup_sdk::build::manifest().service_dir.clone()),
        }
    }

    /// Returns path of the specified service.
    pub async fn get_and_patch(
        &self,
        name: &str,
        patch: Option<PathBuf>,
    ) -> Result<Service, ReadError> {
        let name = name.strip_suffix(".airs").unwrap_or(name);

        let main_path = self
            .base_chain
            .find(format!("{name}.airs"))
            .await
            .ok_or_else(|| ReadError::from(std::io::ErrorKind::NotFound))?;

        let mut paths = Vec::with_capacity(2);
        paths.push(main_path);
        if let Some(path) = patch {
            paths.push(path);
        }

        files::read_merge(paths).await
    }

    /// Lists names of all services installed in the storage.
    pub async fn list(&self) -> Vec<String> {
        self.base_chain
            .read_chain()
            .await
            .map(IntoIterator::into_iter)
            .into_iter()
            .flatten()
            .map(|x| String::from(x.to_string_lossy()))
            .filter(|x| !x.starts_with('.') && x.ends_with(".airs"))
            .map(|x| x.strip_suffix(".airs").unwrap_or(&x).into())
            .collect()
    }
}
impl Default for Services {
    fn default() -> Self {
        Self::new()
    }
}
