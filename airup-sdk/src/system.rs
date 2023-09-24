use super::Error;
use airupfx::{files::Service, prelude::*};
use serde::{Deserialize, Serialize};

/// Representation of the status of a service.
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    /// The service is active.
    Active,

    /// The service has stopped.
    #[default]
    Stopped,
}

/// Result of querying a service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryService {
    pub status: Status,
    pub pid: Option<Pid>,
    pub task: Option<String>,
    pub last_error: Option<Error>,
    pub service: Service,
}
impl QueryService {
    pub fn default_of(service: Service) -> Self {
        Self {
            status: Status::Stopped,
            pid: None,
            task: None,
            last_error: None,
            service,
        }
    }
}

/// Result of querying information about the whole system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySystem {
    pub hostname: Option<String>,
    pub services: Vec<String>,
}

#[async_trait::async_trait]
pub trait ConnectionExt {
    /// Sideloads a service.
    async fn sideload_service(
        &mut self,
        name: &str,
        service: &Service,
    ) -> anyhow::Result<Result<(), Error>>;

    /// Starts the specified service.
    async fn start_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>>;

    /// Stops the specified service.
    async fn stop_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>>;

    /// Reloads the specified service.
    async fn reload_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>>;

    /// Caches the specified service.
    async fn cache_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>>;

    /// Uncaches the specified service.
    async fn uncache_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>>;

    /// Queries the specified service.
    async fn query_service(&mut self, name: &str) -> anyhow::Result<Result<QueryService, Error>>;

    /// Queries information about the whole system.
    async fn query_system(&mut self) -> anyhow::Result<Result<QuerySystem, Error>>;

    /// Refreshes cached system information in the `airupd` daemon.
    async fn refresh(&mut self) -> anyhow::Result<Result<(), Error>>;

    /// Deletes cached system information in the `airupd` daemon.
    async fn gc(&mut self) -> anyhow::Result<Result<(), Error>>;

    /// Powers the system off.
    async fn poweroff(&mut self) -> anyhow::Result<Result<(), Error>>;

    /// Reboots the system.
    async fn reboot(&mut self) -> anyhow::Result<Result<(), Error>>;

    /// Halts the system.
    async fn halt(&mut self) -> anyhow::Result<Result<(), Error>>;
}
#[async_trait::async_trait]
impl<'a> ConnectionExt for super::Connection<'a> {
    async fn sideload_service(
        &mut self,
        name: &str,
        service: &Service,
    ) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.sideload_service", (name, service))
            .await
    }

    async fn start_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.start_service", name).await
    }

    async fn stop_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.stop_service", name).await
    }

    async fn cache_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.cache_service", name).await
    }

    async fn uncache_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.uncache_service", name).await
    }

    async fn reload_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.reload_service", name).await
    }

    async fn query_service(&mut self, name: &str) -> anyhow::Result<Result<QueryService, Error>> {
        self.invoke("system.query_service", name).await
    }

    async fn query_system(&mut self) -> anyhow::Result<Result<QuerySystem, Error>> {
        self.invoke("system.query_system", ()).await
    }

    async fn refresh(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.refresh", ()).await
    }

    async fn gc(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.gc", ()).await
    }

    async fn poweroff(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.poweroff", ()).await
    }

    async fn reboot(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.reboot", ()).await
    }

    async fn halt(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.halt", ()).await
    }
}
