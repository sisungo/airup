use airupfx::{
    files::{ReadError, Service},
    prelude::*,
    sdk::Error,
};
use std::{collections::HashMap, sync::RwLock};

/// Represents to Airup's services directory.
#[derive(Debug)]
pub struct Services {
    base_chain: DirChain,
    sideloaded: RwLock<HashMap<String, Service>>,
}
impl From<DirChain> for Services {
    fn from(val: DirChain) -> Self {
        Self {
            base_chain: val,
            sideloaded: RwLock::default(),
        }
    }
}
impl Services {
    pub fn new() -> Self {
        Self {
            base_chain: DirChain::from(airupfx::config::build_manifest().service_dir.clone()),
            sideloaded: RwLock::default(),
        }
    }

    /// Sideloads a service, fails if the service already exists or is invalid.
    pub fn load(&self, name: &str, mut service: Service) -> Result<(), Error> {
        let name = name.strip_suffix(Service::SUFFIX).unwrap_or(name);
        let mut lock = self.sideloaded.write().unwrap();
        if lock.contains_key(name) {
            return Err(Error::ObjectAlreadyExists);
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
            .ok_or(Error::ObjectNotFound)
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
}