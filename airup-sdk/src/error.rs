use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use thiserror::Error;

/// Represents to an API error.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "code")]
pub enum ApiError {
    /// The client lacks necessary permission to perform the operation.
    #[error("permission denied (requires={requires:?})")]
    PermissionDenied { requires: Vec<String> },

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
    #[error("object already exists")]
    ObjectAlreadyExists,

    /// The requested object was not found.
    #[error("object not found")]
    ObjectNotFound,

    /// The requested object has not been configured yet.
    #[error("object not configured")]
    ObjectNotConfigured,

    /// The requested object is already configured.
    #[error("unit already started")]
    UnitStarted,

    /// The requested object cannot be accessed due to an I/O error.
    #[error("cannot access object: {message}")]
    ObjectIo { message: String },

    /// The requested object format is not considered.
    #[error("invalid object: {message}")]
    InvalidObject { message: Cow<'static, str> },

    /// The requested user was not found.
    #[error("user not found")]
    UserNotFound,

    /// The requested command was not found.
    #[error("command not found")]
    CommandNotFound,

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
    #[error("dependency cannot be satisfied: {name}")]
    DependencyNotSatisfied { name: String },

    /// The operation failed because some conflicts exists.
    #[error("the unit conflicts with a running unit: {name}")]
    ConflictsWith { name: String },

    /// An I/O error occured.
    #[error("I/O error: {message}")]
    Io { message: String },

    /// An internal error.
    #[error("internal error: {message}")]
    Internal { message: Cow<'static, str> },
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

    pub fn permission_denied<R: IntoIterator<Item = S>, S: ToString>(requires: R) -> Self {
        Self::PermissionDenied {
            requires: requires.into_iter().map(|x| x.to_string()).collect(),
        }
    }

    pub fn dependency_not_satisfied<T: Into<String>>(name: T) -> Self {
        Self::DependencyNotSatisfied { name: name.into() }
    }

    pub fn io(err: &std::io::Error) -> Self {
        Self::Io {
            message: err.to_string(),
        }
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
}
impl From<airupfx::files::ReadError> for ApiError {
    fn from(value: airupfx::files::ReadError) -> Self {
        match value {
            airupfx::files::ReadError::Io(err) => match err.kind() {
                std::io::ErrorKind::NotFound => Self::ObjectNotFound,
                _ => Self::ObjectIo {
                    message: err.to_string(),
                },
            },
            airupfx::files::ReadError::Parse(x) => Self::InvalidObject { message: x.into() },
            airupfx::files::ReadError::Validation(x) => Self::InvalidObject { message: x },
        }
    }
}

impl From<airupfx::ace::Error> for ApiError {
    fn from(value: airupfx::ace::Error) -> Self {
        match value {
            airupfx::ace::Error::UserNotFound => Self::UserNotFound,
            airupfx::ace::Error::CommandNotFound => Self::CommandNotFound,
            airupfx::ace::Error::Wait(err) => Self::internal(err.to_string()),
            airupfx::ace::Error::Io(err) => Self::io(&err),
            airupfx::ace::Error::TimedOut => Self::TimedOut,
        }
    }
}
impl From<airupfx::ace::CommandExitError> for ApiError {
    fn from(value: airupfx::ace::CommandExitError) -> Self {
        match value {
            airupfx::ace::CommandExitError::Exited(exit_code) => Self::Exited { exit_code },
            airupfx::ace::CommandExitError::Signaled(signum) => Self::Signaled { signum },
        }
    }
}
