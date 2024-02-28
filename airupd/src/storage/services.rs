//! Represents to Airup's services directory.

use airup_sdk::{
    files::{ReadError, Service, Validate},
    nonblocking::files,
    prelude::*,
    Error,
};
use std::path::PathBuf;
use std::{collections::HashMap, sync::RwLock};

/// Represents to Airup's services directory.
#[derive(Debug)]
pub struct Services {
    base_chain: DirChain<'static>,
    sideloaded: RwLock<HashMap<String, Service>>,
}
impl From<DirChain<'static>> for Services {
    fn from(val: DirChain<'static>) -> Self {
        Self {
            base_chain: val,
            sideloaded: RwLock::default(),
        }
    }
}
impl Services {
    /// Creates a new [`Services`] instance.
    pub fn new() -> Self {
        Self {
            base_chain: DirChain::new(airup_sdk::build::manifest().service_dir.clone()),
            sideloaded: RwLock::default(),
        }
    }

    /// Sideloads a service, fails if the service already exists or is invalid.
    pub fn load(&self, name: &str, mut service: Service, ovrd: bool) -> Result<(), Error> {
        let name = name.strip_suffix(".airs").unwrap_or(name);
        let mut lock = self.sideloaded.write().unwrap();
        if !ovrd && lock.contains_key(name) {
            return Err(Error::UnitExists);
        }
        service.validate()?;
        service.name = name.into();
        lock.insert(name.into(), service);

        Ok(())
    }

    /// Unloads a sideloaded service, fails if the specified service does not exist.
    pub fn unload(&self, name: &str) -> Result<(), Error> {
        let name = name.strip_suffix(".airs").unwrap_or(name);
        self.sideloaded
            .write()
            .unwrap()
            .remove(name)
            .ok_or(Error::UnitNotFound)
            .map(|_| ())
    }

    /// Returns path of the specified service.
    pub async fn get_and_patch(
        &self,
        name: &str,
        patch: Option<PathBuf>,
    ) -> Result<Service, ReadError> {
        let name = name.strip_suffix(".airs").unwrap_or(name);
        if let Some(x) = self.sideloaded.read().unwrap().get(name).cloned() {
            return Ok(x);
        }

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

    /// Lists names of all services installed on the system, including sideloaded ones and on-filesystem ones.
    pub async fn list(&self) -> Vec<String> {
        let mut result = Vec::new();
        self.sideloaded
            .read()
            .unwrap()
            .keys()
            .for_each(|x| result.push(x.to_owned()));
        self.base_chain
            .read_chain()
            .await
            .map(IntoIterator::into_iter)
            .into_iter()
            .flatten()
            .filter(|x| {
                let x = x.to_string_lossy();
                !x.starts_with('.') && x.ends_with(".airs")
            })
            .for_each(|x| {
                let name = x.to_string_lossy();
                let name = name.strip_suffix(".airs").unwrap_or(&name);
                result.push(name.into());
            });
        result
    }
}
impl Default for Services {
    fn default() -> Self {
        Self::new()
    }
}
