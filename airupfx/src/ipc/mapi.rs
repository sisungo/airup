//! # Airup IPC Protocol - Message-level API

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::borrow::Cow;
use thiserror::Error;

/// Represents to an Airup IPC request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub method: String,

    #[serde(alias = "param")]
    pub params: Option<serde_json::Value>,
}
impl Request {
    /// Creates a new [Request] with given method name and parameters.
    pub fn new<M: Into<String>, C: Serialize, P: Into<Option<C>>>(
        method: M,
        params: P,
    ) -> serde_json::Result<Self> {
        let method = method.into();
        let params = params.into().map(|x| serde_json::to_value(x).unwrap());

        Ok(Self { method, params })
    }

    /// Extracts parameters from the request.
    pub fn extract_params<T: DeserializeOwned>(self) -> Result<T, ApiError> {
        let value: serde_json::Value = self.params.into();
        serde_json::from_value(value).map_err(ApiError::invalid_params)
    }
}

/// Represents to an Airup IPC response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "status", content = "payload")]
pub enum Response {
    Ok(serde_json::Value),
    Err(ApiError),
}
impl Response {
    /// Creates a new `Response` from given `Result`.
    ///
    /// ## Panic
    /// Panics when `serde_json::to_value` fails.
    pub fn new<T: Serialize>(result: Result<T, ApiError>) -> Self {
        match result {
            Ok(val) => Self::Ok(serde_json::to_value(&val).unwrap()),
            Err(err) => Self::Err(err),
        }
    }

    /// Converts from `Response` to a `Result`.
    pub fn into_result<T: DeserializeOwned>(self) -> Result<T, ApiError> {
        match self {
            Self::Ok(val) => {
                Ok(serde_json::from_value(val)
                    .map_err(|_| ApiError::bad_response("TypeError", ""))?)
            }
            Self::Err(err) => Err(err),
        }
    }
}

/// Represents to an API error.
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", tag = "code")]
pub enum ApiError {
    /// The client lacks necessary permission to perform the operation.
    #[error("Permission denied (requires={requires:?})")]
    PermissionDenied { requires: Vec<String> },

    /// The requested method was not found.
    #[error("No such method")]
    NoSuchMethod,

    /// The requested method's parameter requirements wasn't satisfied.
    #[error("Invalid parameters: {message}")]
    InvalidParams { message: String },

    /// The request format is not considered.
    #[error("Bad request ({kind}): {message}")]
    BadRequest { kind: String, message: String },

    /// The response format is not considered.
    #[error("Bad response ({kind}): {message}")]
    BadResponse { kind: String, message: String },

    /// The requested object already exists.
    #[error("Object already exists")]
    ObjectAlreadyExists,

    /// The requested object was not found.
    #[error("Object not found")]
    ObjectNotFound,

    /// The requested object has not been configured yet.
    #[error("Object not configured")]
    ObjectNotConfigured,

    /// The requested object is already configured.
    #[error("Object already configured")]
    ObjectAlreadyConfigured,

    /// The requested object cannot be accessed due to an I/O error.
    #[error("Cannot access object: {message}")]
    ObjectIo { message: String },

    /// The requested object format is not considered.
    #[error("Invalid object: {message}")]
    InvalidObject { message: Cow<'static, str> },

    /// The requested user was not found.
    #[error("User not found")]
    UserNotFound,

    /// The requested command was not found.
    #[error("Command not found")]
    CommandNotFound,

    /// The child process unexpectedly exited.
    #[error("Process exited with code {exit_code}")]
    Exited { exit_code: i32 },

    /// The child process was terminated by a signal.
    #[error("Process terminated by signal {signum}")]
    Signaled { signum: i32 },

    /// Failed to read or parse the PID file.
    #[error("Failed to read pidfile: {message}")]
    PidFile { message: String },

    /// There is already a task running.
    #[error("Already exists a task running")]
    TaskAlreadyExists,

    /// The running task is uninterruptable.
    #[error("Task interrupted")]
    TaskInterrupted,

    /// The operation timed out.
    #[error("Operation timed out")]
    TimedOut,

    /// The requested operation in unsupported.
    #[error("Operation not supported: {message}")]
    Unsupported { message: Cow<'static, str> },

    /// The operation failed because some dependencies cannot be satisfied.
    #[error("Dependency cannot be satisfied: {name}")]
    DependencyNotSatisfied { name: String },

    /// An I/O error occured.
    #[error("I/O error: {message}")]
    Io { message: String },

    /// An internal error.
    #[error("Internal error: {message}")]
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
#[cfg(feature = "files")]
impl From<crate::files::ReadError> for ApiError {
    fn from(value: crate::files::ReadError) -> Self {
        match value {
            crate::files::ReadError::Io(err) => match err.kind() {
                std::io::ErrorKind::NotFound => Self::ObjectNotFound,
                _ => Self::ObjectIo {
                    message: err.to_string(),
                },
            },
            crate::files::ReadError::Parse(x) => Self::InvalidObject { message: x.into() },
            crate::files::ReadError::Validation(x) => Self::InvalidObject { message: x },
        }
    }
}
#[cfg(feature = "ace")]
impl From<crate::ace::Error> for ApiError {
    fn from(value: crate::ace::Error) -> Self {
        match value {
            crate::ace::Error::UserNotFound => Self::UserNotFound,
            crate::ace::Error::CommandNotFound => Self::CommandNotFound,
            crate::ace::Error::Wait(err) => Self::internal(err.to_string()),
            crate::ace::Error::Io(err) => Self::io(&err),
            crate::ace::Error::TimedOut => Self::TimedOut,
        }
    }
}
#[cfg(feature = "ace")]
impl From<crate::ace::CommandExitError> for ApiError {
    fn from(value: crate::ace::CommandExitError) -> Self {
        match value {
            crate::ace::CommandExitError::Exited(exit_code) => Self::Exited { exit_code },
            crate::ace::CommandExitError::Signaled(signum) => Self::Signaled { signum },
        }
    }
}
