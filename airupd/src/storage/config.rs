//! Represents to Airup's system config.

use ahash::HashMap;
use airup_sdk::{files::SystemConf, prelude::*};
use std::path::PathBuf;

/// Main navigator of Airup's config directory.
#[derive(Debug)]
pub struct Config {
    pub base_dir: DirChain<'static>,
    pub system_conf: SystemConf,
}
impl Config {
    /// Creates a new [`Config`] instance.
    pub async fn new() -> Self {
        let base_dir = airup_sdk::build::manifest().config_dir.clone();
        let system_conf = SystemConf::read_from(&base_dir.join("system.airc"))
            .await
            .unwrap_or_default();

        Self {
            base_dir: base_dir.into(),
            system_conf,
        }
    }

    /// Returns path of separated config file for specified service.
    pub async fn of_service(&self, name: &str) -> Option<PathBuf> {
        let name = name.strip_suffix(".airs").unwrap_or(name);
        self.base_dir.find(format!("{name}.service.airc")).await
    }

    /// Overrides the environment with the system config.
    pub fn override_env(&self) {
        let mut vars: HashMap<String, Option<String>> = HashMap::default();
        for (k, v) in &airup_sdk::build::manifest().env_vars {
            vars.insert(k.to_owned(), v.as_ref().map(Into::into));
        }
        for (k, v) in &self.system_conf.env.vars {
            vars.insert(k.into(), v.clone());
        }
        airupfx::env::set_vars(vars);
    }
}
