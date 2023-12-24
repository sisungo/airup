//! Error handling

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use thiserror::Error;

/// Represents to an API error.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "code")]
pub enum ApiError {
    /// The requested method was not found.
    #[error("no such method")]
    NoSuchMethod,

    /// The requested method's parameter requirements wasn't satisfied.
    #[error("invalid parameters: {message}")]
    InvalidParams { message: String },

    /// The request format is not considered.
    #[error("bad request ({kind}): {message}")]
    BadRequest { kind: String, message: String },

    /// The response format is not considered.
    #[error("bad response ({kind}): {message}")]
    BadResponse { kind: String, message: String },

    /// The requested object already exists.
    #[error("unit already exists")]
    UnitExists,

    /// The requested object was not found.
    #[error("unit not found")]
    UnitNotFound,

    /// The requested object has not been configured yet.
    #[error("unit not started")]
    UnitNotStarted,

    /// The requested object is already configured.
    #[error("unit already started")]
    UnitStarted,

    /// The requested object format is not considered.
    #[error("unit ill formatted: {message}")]
    BadUnit { message: Cow<'static, str> },

    /// The requested user was not found.
    #[error("user not found")]
    UserNotFound,

    /// The requested command was not found.
    #[error("command not found")]
    CommandNotFound,

    /// The watchdog barked.
    #[error("watchdog barked")]
    Watchdog,

    /// The child process unexpectedly exited.
    #[error("process exited with code {exit_code}")]
    Exited { exit_code: i32 },

    /// The child process was terminated by a signal.
    #[error("process terminated by signal {signum}")]
    Signaled { signum: i32 },

    /// Failed to read or parse the PID file.
    #[error("failed to read pidfile: {message}")]
    PidFile { message: String },

    /// There is already a task running.
    #[error("already exists a task running")]
    TaskExists,

    /// There is no task running.
    #[error("task not found")]
    TaskNotFound,

    /// The running task is uninterruptable.
    #[error("task interrupted")]
    TaskInterrupted,

    /// The operation timed out.
    #[error("operation timed out")]
    TimedOut,

    /// The requested operation in unsupported.
    #[error("operation not supported: {message}")]
    Unsupported { message: Cow<'static, str> },

    /// The operation failed because some dependencies cannot be satisfied.
    #[error("dependency `{name}` cannot be satisfied")]
    DepNotSatisfied { name: String },

    /// The operation failed because some conflicts exists.
    #[error("the unit conflicts with unit `{name}`")]
    ConflictsWith { name: String },

    /// ACE parse error.
    #[error("ace: parse error")]
    AceParseError,

    /// An I/O error occured.
    #[error("I/O error: {message}")]
    Io { message: String },

    /// An internal error.
    #[error("internal error: {message}")]
    Internal { message: Cow<'static, str> },

    /// A custom error.
    #[error("{message}")]
    Custom { message: String },
}
impl ApiError {
    pub fn bad_request<K: Into<String>, M: Into<String>>(kind: K, message: M) -> Self {
        Self::BadRequest {
            kind: kind.into(),
            message: message.into(),
        }
    }

    pub fn bad_response<K: Into<String>, M: Into<String>>(kind: K, message: M) -> Self {
        Self::BadResponse {
            kind: kind.into(),
            message: message.into(),
        }
    }

    pub fn missing_params() -> Self {
        Self::invalid_params("missing parameters")
    }

    pub fn invalid_params<E: ToString>(err: E) -> Self {
        Self::InvalidParams {
            message: err.to_string(),
        }
    }

    pub fn dep_not_satisfied<T: Into<String>>(name: T) -> Self {
        Self::DepNotSatisfied { name: name.into() }
    }

    pub fn unsupported<T: Into<Cow<'static, str>>>(message: T) -> Self {
        Self::Unsupported {
            message: message.into(),
        }
    }

    pub fn internal<T: Into<Cow<'static, str>>>(message: T) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    pub fn pid_file<T: ToString>(err: T) -> Self {
        Self::PidFile {
            message: err.to_string(),
        }
    }

    pub fn custom<T: ToString>(err: T) -> Self {
        Self::Custom {
            message: err.to_string(),
        }
    }
}

pub trait IntoApiError {
    fn into_api_error(self) -> ApiError;
}

impl<T: IntoApiError> From<T> for ApiError {
    fn from(value: T) -> Self {
        value.into_api_error()
    }
}
