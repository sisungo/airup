use crate::{files::Service, Error};
use serde::{Deserialize, Serialize};

/// Result of querying a service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryService {
    pub status: Status,
    pub status_since: Option<i64>,
    pub pid: Option<i64>,
    pub memory_usage: Option<u64>,
    pub task_class: Option<String>,
    pub last_error: Option<Error>,
    pub definition: Service,
}
impl QueryService {
    pub fn default_of(definition: Service) -> Self {
        Self {
            status: Status::Stopped,
            status_since: None,
            pid: None,
            memory_usage: None,
            task_class: None,
            last_error: None,
            definition,
        }
    }
}

/// Result of querying information about the whole system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySystem {
    /// Status of the system.
    pub status: Status,

    /// Timestamp generated when the system started to boot.
    pub boot_timestamp: i64,

    /// Timestamp generated when the system completed booting.
    pub booted_since: Option<i64>,

    /// Indicates whether the system is booting.
    pub is_booting: bool,

    /// List of entered milestones in the system.
    pub milestones: Vec<EnteredMilestone>,

    /// Hostname of the system.
    pub hostname: Option<String>,

    /// List of cached services in the system.
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
    /// Timestamp generated when the log record is emitted.
    pub timestamp: i64,

    /// Module of the log record.
    pub module: String,

    /// Message of the log record.
    pub message: String,
}

/// Information of an entered milestone.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnteredMilestone {
    /// Name of the milestone.
    pub name: String,

    /// Timestamp generated when we started to enter the milestone.
    pub begin_timestamp: i64,

    /// Timestamp generated when we completed entering the milestone.
    pub finish_timestamp: i64,
}

/// Representation of an Airup event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Identifier of the event.
    pub id: String,

    /// Payload data of the event.
    pub payload: String,
}
impl Event {
    /// Creates a new [`Event`] instance with given ID and paylaod.
    pub fn new(id: String, payload: String) -> Self {
        Self { id, payload }
    }
}

/// An extension trait to provide `system.*` API invocation.
pub trait ConnectionExt<'a>: crate::Connection {
    /// Sideloads a service.
    fn sideload_service(
        &'a mut self,
        name: &'a str,
        service: &'a Service,
        ovrd: bool,
    ) -> Self::Invoke<'a, ()> {
        self.invoke("system.sideload_service", (name, service, ovrd))
    }

    /// Starts the specified service.
    fn start_service(&'a mut self, name: &'a str) -> Self::Invoke<'a, ()> {
        self.invoke("system.start_service", name)
    }

    /// Stops the specified service.
    fn stop_service(&'a mut self, name: &'a str) -> Self::Invoke<'a, ()> {
        self.invoke("system.stop_service", name)
    }

    /// Forces the specified service to stop.
    fn kill_service(&'a mut self, name: &'a str) -> Self::Invoke<'a, ()> {
        self.invoke("system.kill_service", name)
    }

    /// Reloads the specified service.
    fn reload_service(&'a mut self, name: &'a str) -> Self::Invoke<'a, ()> {
        self.invoke("system.reload_service", name)
    }

    /// Caches the specified service.
    fn cache_service(&'a mut self, name: &'a str) -> Self::Invoke<'a, ()> {
        self.invoke("system.cache_service", name)
    }

    /// Uncaches the specified service.
    fn uncache_service(&'a mut self, name: &'a str) -> Self::Invoke<'a, ()> {
        self.invoke("system.uncache_service", name)
    }

    /// Queries the specified service.
    fn query_service(&'a mut self, name: &'a str) -> Self::Invoke<'a, QueryService> {
        self.invoke("system.query_service", name)
    }

    /// Interrupts current task running in specific service's supervisor.
    fn interrupt_service_task(&'a mut self, name: &'a str) -> Self::Invoke<'a, ()> {
        self.invoke("system.interrupt_service_task", name)
    }

    /// Queries information about the whole system.
    fn query_system(&'a mut self) -> Self::Invoke<'a, QuerySystem> {
        self.invoke("system.query_system", ())
    }

    /// Lists all services.
    fn list_services(&'a mut self) -> Self::Invoke<'a, Vec<String>> {
        self.invoke("system.list_services", ())
    }

    /// Refreshes cached system information in the `airupd` daemon.
    fn refresh(&'a mut self) -> Self::Invoke<'a, ()> {
        self.invoke("system.refresh", ())
    }

    /// Deletes cached system information in the `airupd` daemon.
    fn gc(&'a mut self) -> Self::Invoke<'a, ()> {
        self.invoke("system.gc", ())
    }

    /// Enters the specific milestone.
    fn enter_milestone(&'a mut self, name: &'a str) -> Self::Invoke<'a, ()> {
        self.invoke("system.enter_milestone", name)
    }

    /// Triggers the specific event.
    fn trigger_event(&'a mut self, event: &'a Event) -> Self::Invoke<'a, ()> {
        self.invoke("system.trigger_event", event)
    }

    /// Loads an extension.
    fn load_extension(&'a mut self, name: &'a str, path: &'a str) -> Self::Invoke<'a, ()> {
        self.invoke("system.load_extension", (name, path))
    }

    /// Unloads an extension.
    fn unload_extension(&'a mut self, name: &'a str) -> Self::Invoke<'a, ()> {
        self.invoke("system.load_extension", name)
    }
}
impl<'a, T> ConnectionExt<'a> for T where T: crate::Connection {}
