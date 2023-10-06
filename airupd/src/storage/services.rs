use airup_sdk::files::{ReadError, Service};
use airup_sdk::Error;
use airupfx::prelude::*;
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
    pub fn new() -> Self {
        Self {
            base_chain: DirChain::new(airupfx::config::BUILD_MANIFEST.service_dir),
            sideloaded: RwLock::default(),
        }
    }

    /// Sideloads a service, fails if the service already exists or is invalid.
    pub fn load(&self, name: &str, mut service: Service) -> Result<(), Error> {
        let name = name.strip_suffix(Service::SUFFIX).unwrap_or(name);
        let mut lock = self.sideloaded.write().unwrap();
        if lock.contains_key(name) {
            return Err(Error::UnitExists);
        }
        service.validate()?;
        service.name = name.into();
        lock.insert(name.into(), service);

        Ok(())
    }

    /// Unloads a sideloaded service, fails if the specified service does not exist.
    pub fn unload(&self, name: &str) -> Result<(), Error> {
        let name = name.strip_suffix(Service::SUFFIX).unwrap_or(name);
        self.sideloaded
            .write()
            .unwrap()
            .remove(name)
            .ok_or(Error::UnitNotFound)
            .map(|_| ())
    }

    /// Attempts to find and parse a service.
    pub async fn get(&self, name: &str) -> Result<Service, ReadError> {
        let name = name.strip_suffix(Service::SUFFIX).unwrap_or(name);
        if let Some(x) = self.sideloaded.read().unwrap().get(name).cloned() {
            return Ok(x);
        }
        match self
            .base_chain
            .find(format!("{name}{}", Service::SUFFIX))
            .await
        {
            Some(x) => Service::read_from(x).await,
            None => Err(std::io::ErrorKind::NotFound.into()),
        }
    }

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
            .for_each(|x| {
                let name = x.to_string_lossy();
                let name = name.strip_suffix(Service::SUFFIX).unwrap_or(&name);
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
