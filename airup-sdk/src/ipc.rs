//! # Airup IPC Protocol
//! Airup's protocol for IPC.
//!
//! ## Stream to Datagram
//! Airup uses a very simple protocol to wrap streaming protocols to datagram protocols.
//!
//! ### The Conversation Model
//! The simple protocol is a half-duplex, synchronous "blocking" protocol. The client sends a request and the server sends a
//! response. If an serious protocol error occured, the connection will be simply shut down.
//!
//! ### Datagram Layout
//! A datagram begins with a 64-bit long, little-endian ordered integer, which represents to the datagram's length. The length
//! should be less than 6MiB and cannot be zero, or a serious protocol error will be occured. Then follows content of the
//! datagram.

use crate::error::ApiError;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// A request object in the Airup IPC protocol.
///
/// Interpreted as CBOR, a serialized request object looks like (converted to human-readable JSON):
///
/// ```json
/// {
///     "status": "<ok | err>",
///     "payload": <payload>,
/// }
/// ```
///
/// If the method requires no parameters, `params` can be `null` or even not present. If the method requires only one parameter,
/// the field is the parameter itself. If the method requires more than one parameters, the field is an array filled with
/// parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub method: String,

    #[serde(alias = "param")]
    pub params: Option<ciborium::Value>,
}
impl Request {
    /// Creates a new [`Request`] with given method name and parameters.
    pub fn new<M: Into<String>, C: Serialize, P: Into<Option<C>>>(method: M, params: P) -> Self {
        let method = method.into();
        let params = params
            .into()
            .map(|x| ciborium::Value::serialized(&x).unwrap());

        Self { method, params }
    }

    /// Extracts parameters from the request.
    pub fn extract_params<T: DeserializeOwned>(self) -> Result<T, ApiError> {
        let value = self.params.unwrap_or(ciborium::Value::Null);
        value.deserialized().map_err(ApiError::invalid_params)
    }
}

/// A response object in the Airup IPC protocol.
///
/// Interpreted as CBOR, a serialized response object looks like (converted to human-readable JSON):
///
/// ```json
/// {
///     "status": "<ok | err>",
///     "payload": <payload>,
/// }
/// ```
///
/// On success, `status` is `ok`, and `payload` is the return value of the requested method.
///
/// On failure, `status` is `err`, and `payload` is an [`Error`] object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "status", content = "payload")]
pub enum Response {
    Ok(ciborium::Value),
    Err(ApiError),
}
impl Response {
    /// Creates a new `Response` from given `Result`.
    ///
    /// # Panics
    /// Panics when CBOR serialization fails. This always assumes that the passed value is always interpreted as a value
    /// CBOR object.
    pub fn new<T: Serialize>(result: Result<T, ApiError>) -> Self {
        match result {
            Ok(val) => Self::Ok(ciborium::Value::serialized(&val).unwrap()),
            Err(err) => Self::Err(err),
        }
    }

    /// Converts from `Response` to a `Result`.
    pub fn into_result<T: DeserializeOwned>(self) -> Result<T, ApiError> {
        match self {
            Self::Ok(val) => Ok(val
                .deserialized()
                .map_err(|err| ApiError::bad_response("TypeError", format!("{:?}", err)))?),
            Self::Err(err) => Err(err),
        }
    }
}

/// A middle layer that splits a stream into messages.
#[derive(Debug)]
pub struct MessageProto<T> {
    pub(crate) inner: T,
    pub(crate) size_limit: usize,
}
impl<T> MessageProto<T> {
    pub const DEFAULT_SIZE_LIMIT: usize = 6 * 1024 * 1024;

    /// Creates a new [`MessageProto`] with provided stream.
    pub fn new(inner: T, size_limit: usize) -> Self {
        Self { inner, size_limit }
    }

    /// Sets received message size limitation.
    pub fn set_size_limit(&mut self, new: usize) -> usize {
        std::mem::replace(&mut self.size_limit, new)
    }

    /// Gets received message size limitation.
    pub fn size_limit(&self) -> usize {
        self.size_limit
    }
}
impl<T> AsRef<T> for MessageProto<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}
impl<T> AsMut<T> for MessageProto<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}
impl<T> From<T> for MessageProto<T> {
    fn from(value: T) -> Self {
        Self::new(value, Self::DEFAULT_SIZE_LIMIT)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Io(std::io::Error),

    #[error("{0}")]
    CborSer(ciborium::ser::Error<std::io::Error>),

    #[error("{0}")]
    CborDe(ciborium::de::Error<std::io::Error>),

    #[error("message too long: {0} bytes")]
    MessageTooLong(usize),
}
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<ciborium::de::Error<std::io::Error>> for Error {
    fn from(value: ciborium::de::Error<std::io::Error>) -> Self {
        Self::CborDe(value)
    }
}
impl From<ciborium::ser::Error<std::io::Error>> for Error {
    fn from(value: ciborium::ser::Error<std::io::Error>) -> Self {
        Self::CborSer(value)
    }
}
