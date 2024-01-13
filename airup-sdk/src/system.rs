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
