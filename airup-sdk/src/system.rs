use super::Error;
use crate::files::Service;
use airupfx::prelude::*;
use duplicate::duplicate_item;
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
    pub status_since: Option<i64>,
    pub pid: Option<Pid>,
    pub task: Option<String>,
    pub last_error: Option<Error>,
    pub service: Service,
}
impl QueryService {
    pub fn default_of(service: Service) -> Self {
        Self {
            status: Status::Stopped,
            status_since: None,
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
    pub status: Status,
    pub status_since: i64,
    pub hostname: Option<String>,
    pub services: Vec<String>,
}

#[duplicate_item(
    Name                       async;
    [ConnectionExt]            [async];
    [BlockingConnectionExt]    [];
)]
#[async_trait::async_trait]
pub trait Name {
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

    /// Lists all services.
    async fn list_services(&mut self) -> anyhow::Result<Result<Vec<String>, Error>>;

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
#[duplicate_item(
    Name                       async      Connection                         may_await(code);
    [ConnectionExt]            [async]    [super::Connection<'_>]            [code.await];
    [BlockingConnectionExt]    []         [super::BlockingConnection<'_>]    [code];
)]
#[async_trait::async_trait]
impl Name for Connection {
    async fn sideload_service(
        &mut self,
        name: &str,
        service: &Service,
    ) -> anyhow::Result<Result<(), Error>> {
        may_await([self.invoke("system.sideload_service", (name, service))])
    }

    async fn start_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        may_await([self.invoke("system.start_service", name)])
    }

    async fn stop_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        may_await([self.invoke("system.stop_service", name)])
    }

    async fn cache_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        may_await([self.invoke("system.cache_service", name)])
    }

    async fn uncache_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        may_await([self.invoke("system.uncache_service", name)])
    }

    async fn reload_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        may_await([self.invoke("system.reload_service", name)])
    }

    async fn query_service(&mut self, name: &str) -> anyhow::Result<Result<QueryService, Error>> {
        may_await([self.invoke("system.query_service", name)])
    }

    async fn list_services(&mut self) -> anyhow::Result<Result<Vec<String>, Error>> {
        may_await([self.invoke("system.list_services", ())])
    }

    async fn query_system(&mut self) -> anyhow::Result<Result<QuerySystem, Error>> {
        may_await([self.invoke("system.query_system", ())])
    }

    async fn refresh(&mut self) -> anyhow::Result<Result<(), Error>> {
        may_await([self.invoke("system.refresh", ())])
    }

    async fn gc(&mut self) -> anyhow::Result<Result<(), Error>> {
        may_await([self.invoke("system.gc", ())])
    }

    async fn poweroff(&mut self) -> anyhow::Result<Result<(), Error>> {
        may_await([self.invoke("system.poweroff", ())])
    }

    async fn reboot(&mut self) -> anyhow::Result<Result<(), Error>> {
        may_await([self.invoke("system.reboot", ())])
    }

    async fn halt(&mut self) -> anyhow::Result<Result<(), Error>> {
        may_await([self.invoke("system.halt", ())])
    }
}
