use super::Error;
use crate::files::Service;
use serde::{Deserialize, Serialize};
use std::future::Future;

/// Result of querying a service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryService {
    pub status: Status,
    pub status_since: Option<i64>,
    pub pid: Option<i64>,
    pub task_class: Option<String>,
    pub task_name: Option<String>,
    pub last_error: Option<Error>,
    pub definition: Service,
}
impl QueryService {
    pub fn default_of(definition: Service) -> Self {
        Self {
            status: Status::Stopped,
            status_since: None,
            pid: None,
            task_class: None,
            task_name: None,
            last_error: None,
            definition,
        }
    }
}

/// Result of querying information about the whole system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySystem {
    pub status: Status,
    pub boot_timestamp: i64,
    pub booted_since: Option<i64>,
    pub is_booting: bool,
    pub hostname: Option<String>,
    pub services: Vec<String>,
}

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

/// Item of an log record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRecord {
    pub timestamp: i64,
    pub module: String,
    pub message: String,
}

/// Information of an entered milestone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnteredMilestone {
    pub name: String,
    pub begin_timestamp: i64,
    pub finish_timestamp: i64,
}

pub trait ConnectionExt {
    /// Sideloads a service.
    fn sideload_service(
        &mut self,
        name: &str,
        service: &Service,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>> + Send;

    /// Starts the specified service.
    fn start_service(
        &mut self,
        name: &str,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Stops the specified service.
    fn stop_service(
        &mut self,
        name: &str,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Reloads the specified service.
    fn reload_service(
        &mut self,
        name: &str,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Caches the specified service.
    fn cache_service(
        &mut self,
        name: &str,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Uncaches the specified service.
    fn uncache_service(
        &mut self,
        name: &str,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Queries the specified service.
    fn query_service(
        &mut self,
        name: &str,
    ) -> impl Future<Output = anyhow::Result<Result<QueryService, Error>>>;

    /// Interrupts current task running in specific service's supervisor.
    fn interrupt_service_task(
        &mut self,
        name: &str,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Queries information about the whole system.
    fn query_system(&mut self) -> impl Future<Output = anyhow::Result<Result<QuerySystem, Error>>>;

    /// Lists all services.
    fn list_services(&mut self)
        -> impl Future<Output = anyhow::Result<Result<Vec<String>, Error>>>;

    /// Refreshes cached system information in the `airupd` daemon.
    fn refresh(&mut self) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Deletes cached system information in the `airupd` daemon.
    fn gc(&mut self) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Powers the system off.
    fn poweroff(&mut self) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Reboots the system.
    fn reboot(&mut self) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Halts the system.
    fn halt(&mut self) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Indicates `airupd` to register the specified logger.
    fn use_logger(&mut self, name: &str)
        -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Queries latest `n` log records from the logger.
    fn tail_logs(
        &mut self,
        subject: &str,
        n: usize,
    ) -> impl Future<Output = anyhow::Result<Result<Vec<LogRecord>, Error>>>;

    /// Enters the specific milestone.
    fn enter_milestone(
        &mut self,
        name: &str,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;
}
impl ConnectionExt for super::Connection {
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

    async fn interrupt_service_task(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.interrupt_service_task", name).await
    }

    async fn list_services(&mut self) -> anyhow::Result<Result<Vec<String>, Error>> {
        self.invoke("system.list_services", ()).await
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

    async fn use_logger(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.use_logger", name).await
    }

    async fn tail_logs(
        &mut self,
        subject: &str,
        n: usize,
    ) -> anyhow::Result<Result<Vec<LogRecord>, Error>> {
        self.invoke("system.tail_logs", (subject, n)).await
    }

    async fn enter_milestone(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.enter_milestone", name).await
    }
}
