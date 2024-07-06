//! Represents to Airup's config directory.

use airup_sdk::{files::SystemConf, prelude::*};
use std::{collections::HashMap, path::PathBuf};

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
        self.base_dir.find(format!("{name}.airs.airc")).await
    }

    /// Populates the process' environment with the system config.
    pub fn populate_system_config(&self) {
        if !self.system_conf.system.instance_name.is_empty() {
            airupfx::env::set_instance_name(self.system_conf.system.instance_name.clone());
        }

        let mut vars: HashMap<String, Option<String>> = HashMap::default();
        for (k, v) in &airup_sdk::build::manifest().env_vars {
            vars.insert(k.to_owned(), v.as_ref().map(Into::into));
        }
        for (k, v) in &self.system_conf.env.vars {
            if v.as_integer() == Some(0) {
                vars.insert(k.clone(), None);
            } else if let Some(s) = v.as_str() {
                vars.insert(k.clone(), Some(s.into()));
            }
        }
        airupfx::env::set_vars(vars);
    }
}
