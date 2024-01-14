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

use crate::Error;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub const DEFAULT_SIZE_LIMIT: usize = 6 * 1024 * 1024;

/// A request object in the Airup IPC protocol.
///
/// Interpreted as JSON, a serialized request object looks like:
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
    pub params: Option<serde_json::Value>,
}
impl Request {
    /// Creates a new [`Request`] with given method name and parameters.
    pub fn new<M: Into<String>, C: Serialize, P: Into<Option<C>>>(
        method: M,
        params: P,
    ) -> serde_json::Result<Self> {
        let method = method.into();
        let params = params.into().map(|x| serde_json::to_value(x).unwrap());

        Ok(Self { method, params })
    }

    /// Extracts parameters from the request.
    pub fn extract_params<T: DeserializeOwned>(self) -> Result<T, Error> {
        let value: serde_json::Value = self.params.into();
        serde_json::from_value(value).map_err(Error::invalid_params)
    }
}

/// A response object in the Airup IPC protocol.
///
/// Interpreted as JSON, a serialized response object looks like:
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
    Ok(serde_json::Value),
    Err(Error),
}
impl Response {
    /// Creates a new `Response` from given `Result`.
    ///
    /// # Panics
    /// Panics when `serde_json::to_value` fails. This always assumes that the passed value is always interpreted as a value
    /// JSON object.
    pub fn new<T: Serialize>(result: Result<T, Error>) -> Self {
        match result {
            Ok(val) => Self::Ok(serde_json::to_value(&val).unwrap()),
            Err(err) => Self::Err(err),
        }
    }

    /// Converts from `Response` to a `Result`.
    pub fn into_result<T: DeserializeOwned>(self) -> Result<T, Error> {
        match self {
            Self::Ok(val) => Ok(serde_json::from_value(val)
                .map_err(|err| Error::bad_response("TypeError", format!("{:?}", err)))?),
            Self::Err(err) => Err(err),
        }
    }
}
