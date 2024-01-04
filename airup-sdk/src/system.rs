use crate::{files::Service, Error};
use serde::{Deserialize, Serialize};

/// Result of querying a service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryService {
    pub status: Status,
    pub status_since: Option<i64>,
    pub pid: Option<i64>,
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
            task_class: None,
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
    pub milestones: Vec<EnteredMilestone>,
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
