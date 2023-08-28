//! Represents to Airup's config directory.

use airupfx::{
    files::{Milestone, ReadError, Service},
    policy,
    prelude::*,
    sdk::Error,
};
use std::{collections::HashMap, sync::RwLock};

/// Main navigator of Airup's config directory.
#[derive(Debug)]
pub struct Config {
    pub services: Services,
    pub milestones: Milestones,
    pub policy: ConcurrentInit<policy::Db>,
}
impl Config {
    /// Creates a new [Config] instance.
    #[inline]
    pub fn new() -> Self {
        let base_dir = &airupfx::config::build_manifest().config_dir;

        Self {
            services: DirChain::from(base_dir.join("services")).into(),
            milestones: DirChain::from(base_dir.join("milestones")).into(),
            policy: ConcurrentInit::new(policy::Db::new(DirChain::from(base_dir.join("policy")))),
        }
    }
}
impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

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

/// Represents to Airup's milestones directory.
#[derive(Debug)]
pub struct Milestones {
    base_chain: DirChain,
}
impl From<DirChain> for Milestones {
    fn from(val: DirChain) -> Self {
        Self { base_chain: val }
    }
}
impl Milestones {
    /// Attempts to find and parse a milestone.
    pub async fn get(&self, name: &str) -> Result<Milestone, ReadError> {
        let name = name.strip_suffix(Milestone::SUFFIX).unwrap_or(name);
        match self
            .base_chain
            .find(format!("{name}{}", Milestone::SUFFIX))
            .await
        {
            Some(x) => Milestone::read_from(x).await,
            None => Err(std::io::ErrorKind::NotFound.into()),
        }
    }
}
