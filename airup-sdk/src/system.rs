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
pub struct QueryResult {
    pub status: Status,
    pub pid: Option<Pid>,
    pub task: Option<String>,
    pub last_error: Option<Error>,
    pub service: Service,
}
impl QueryResult {
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

    /// Queries the specified service.
    async fn query_service(&mut self, name: &str) -> anyhow::Result<Result<QueryResult, Error>>;

    /// Gets a list of all running supervisors.
    async fn supervisors(&mut self) -> anyhow::Result<Result<Vec<String>, Error>>;

    /// Shuts the system down.
    async fn shutdown(&mut self) -> anyhow::Result<Result<(), Error>>;

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

    async fn reload_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.reload_service", name).await
    }

    async fn query_service(&mut self, name: &str) -> anyhow::Result<Result<QueryResult, Error>> {
        self.invoke("system.query_service", name).await
    }

    async fn supervisors(&mut self) -> anyhow::Result<Result<Vec<String>, Error>> {
        self.invoke("system.query_service", ()).await
    }

    async fn shutdown(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.shutdown", ()).await
    }

    async fn reboot(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.reboot", ()).await
    }

    async fn halt(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.halt", ()).await
    }
}
