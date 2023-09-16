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
use anyhow::anyhow;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    ops::{Deref, DerefMut},
    path::Path,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{unix::UCred, UnixListener, UnixStream},
};

/// Represents to a connection.
#[derive(Debug)]
pub struct Connection(S2D<UnixStream>);
impl Connection {
    /// Connects to the specified socket.
    pub async fn connect<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        Ok(Self(UnixStream::connect(path).await?.into()))
    }

    /// Receives a datagram and deserializes it from JSON to `T`.
    pub async fn recv<T: DeserializeOwned>(&mut self) -> anyhow::Result<T> {
        Ok(serde_json::from_slice(&self.0.recv().await?)?)
    }

    /// Receives a request.
    pub async fn recv_req(&mut self) -> anyhow::Result<Request> {
        let req: Request = serde_json::from_slice(&self.0.recv().await?).unwrap_or_else(|err| {
            Request::new(
                "debug.echo_raw",
                Response::Err(ApiError::bad_request("InvalidJson", err.to_string())),
            )
            .unwrap()
        });
        Ok(req)
    }

    /// Receives a response.
    pub async fn recv_resp(&mut self) -> anyhow::Result<Response> {
        self.recv().await
    }

    /// Sends a datagram with JSON-serialized given object.
    pub async fn send<T: Serialize>(&mut self, obj: &T) -> anyhow::Result<()> {
        self.0.send(serde_json::to_string(obj)?.as_bytes()).await
    }

    /// Attempts to get peer's UNIX credentials.
    pub fn cred(&self) -> std::io::Result<UCred> {
        self.0.as_ref().peer_cred()
    }
}
impl Deref for Connection {
    type Target = S2D<UnixStream>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Connection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A wrap of `UnixListener` that accepts [Connection].
#[derive(Debug)]
pub struct Server(UnixListener);
impl Server {
    /// Creates a new instance, binding to the given path.
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        Ok(Self(UnixListener::bind(path)?))
    }

    /// Accepts an connection.
    pub async fn accept(&self) -> std::io::Result<Connection> {
        Ok(Connection(self.0.accept().await?.0.into()))
    }
}

/// Representation of an Airup IPC request.
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

/// A connection.
#[derive(Debug)]
pub struct S2D<T>(T);
impl<T> S2D<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    /// Maximum length of a datagram.
    pub const MAX_DATAGRAM_LEN: usize = 6 * 1024 * 1024;

    /// Receives a datagram.
    pub async fn recv(&mut self) -> anyhow::Result<Vec<u8>> {
        let len = self.0.read_u64_le().await? as usize;
        if len > Self::MAX_DATAGRAM_LEN {
            return Err(anyhow!("datagram is too big ({} bytes)", len));
        }
        let mut blob = vec![0u8; len];
        self.0.read_exact(&mut blob).await?;

        Ok(blob)
    }

    /// Sends a datagram.
    pub async fn send(&mut self, blob: &[u8]) -> anyhow::Result<()> {
        if blob.len() > Self::MAX_DATAGRAM_LEN {
            return Err(anyhow!("datagram is too big ({} bytes)", blob.len()));
        }
        self.0.write_u64_le(blob.len() as _).await?;
        self.0.write_all(blob).await?;

        Ok(())
    }

    /// Creates a new `Connection` with provided stream.
    pub fn new(stream: T) -> Self {
        stream.into()
    }
}
impl<T> AsRef<T> for S2D<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
impl<T> AsMut<T> for S2D<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
impl<T> From<T> for S2D<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}
