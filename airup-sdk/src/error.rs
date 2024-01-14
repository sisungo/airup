//! Airup SDK error types.

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
    pub fn bad_request(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Self::BadRequest {
            kind: kind.into(),
            message: message.into(),
        }
    }

    pub fn bad_response(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Self::BadResponse {
            kind: kind.into(),
            message: message.into(),
        }
    }

    pub fn missing_params() -> Self {
        Self::invalid_params("missing parameters")
    }

    pub fn invalid_params(err: impl ToString) -> Self {
        Self::InvalidParams {
            message: err.to_string(),
        }
    }

    pub fn dep_not_satisfied(name: impl Into<String>) -> Self {
        Self::DepNotSatisfied { name: name.into() }
    }

    pub fn unsupported(message: impl Into<Cow<'static, str>>) -> Self {
        Self::Unsupported {
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<Cow<'static, str>>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    pub fn pid_file(err: impl ToString) -> Self {
        Self::PidFile {
            message: err.to_string(),
        }
    }

    pub fn custom(err: impl ToString) -> Self {
        Self::Custom {
            message: err.to_string(),
        }
    }
}

/// A trait that hints the error type may be converted into [`ApiError`].
pub trait IntoApiError {
    /// Convert from the error type into [`ApiError`].
    fn into_api_error(self) -> ApiError;
}
impl<T: IntoApiError> From<T> for ApiError {
    fn from(value: T) -> Self {
        value.into_api_error()
    }
}
