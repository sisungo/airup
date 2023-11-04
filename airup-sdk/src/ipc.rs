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
use duplicate::duplicate_item;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    io::{Read, Write},
    ops::{Deref, DerefMut},
    path::Path,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::UnixListener,
};

/// Represents to a connection.
#[duplicate_item(
    Name                    Stream;
    [Connection]            [tokio::net::UnixStream];
    [BlockingConnection]    [std::os::unix::net::UnixStream];
)]
#[derive(Debug)]
pub struct Name(S2D<Stream>);
#[duplicate_item(
    Name                     Stream                              async      may_await(code)    receive(code)             send_to(who, blob);
    [Connection]             [tokio::net::UnixStream]            [async]    [code.await]       [code.recv().await]       [who.send(blob).await];
    [BlockingConnection]     [std::os::unix::net::UnixStream]    []         [code]             [code.recv_blocking()]    [who.send_blocking(blob)];
)]
impl Name {
    /// Connects to the specified socket.
    pub async fn connect<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        Ok(Self(S2D::new(
            may_await([Stream::connect(path)])?,
            usize::MAX,
        )))
    }

    /// Receives a datagram and deserializes it from JSON to `T`.
    pub async fn recv<T: DeserializeOwned>(&mut self) -> anyhow::Result<T> {
        Ok(serde_json::from_slice(&receive([self.0])?)?)
    }

    /// Receives a request from the underlying protocol.
    pub async fn recv_req(&mut self) -> anyhow::Result<Request> {
        let req: Request = serde_json::from_slice(&receive([self.0])?).unwrap_or_else(|err| {
            Request::new(
                "debug.echo_raw",
                Response::Err(ApiError::bad_request("InvalidJson", err.to_string())),
            )
            .unwrap()
        });
        Ok(req)
    }

    /// Receives a response from the underlying protocol.
    pub async fn recv_resp(&mut self) -> anyhow::Result<Response> {
        may_await([self.recv()])
    }

    /// Sends a datagram with JSON-serialized given object.
    pub async fn send<T: Serialize>(&mut self, obj: &T) -> anyhow::Result<()> {
        send_to([self.0], [serde_json::to_string(obj)?.as_bytes()])
    }
}
#[duplicate_item(
    Name                    Stream;
    [Connection]            [tokio::net::UnixStream];
    [BlockingConnection]    [std::os::unix::net::UnixStream];
)]
impl Deref for Name {
    type Target = S2D<Stream>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[duplicate_item(Name; [Connection]; [BlockingConnection];)]
impl DerefMut for Name {
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
    /// # Panics
    /// Panics when `serde_json::to_value` fails. This always assumes that the passed value is always interpreted as a value
    /// JSON object.
    pub fn new<T: Serialize>(result: Result<T, ApiError>) -> Self {
        match result {
            Ok(val) => Self::Ok(serde_json::to_value(&val).unwrap()),
            Err(err) => Self::Err(err),
        }
    }

    /// Converts from `Response` to a `Result`.
    pub fn into_result<T: DeserializeOwned>(self) -> Result<T, ApiError> {
        match self {
            Self::Ok(val) => Ok(serde_json::from_value(val)
                .map_err(|err| ApiError::bad_response("TypeError", format!("{:?}", err)))?),
            Self::Err(err) => Err(err),
        }
    }
}

/// A connection.
#[derive(Debug)]
pub struct S2D<T> {
    inner: T,
    size_limit: usize,
}
impl<T> S2D<T> {
    /// Sets received datagram size limitation.
    pub fn set_size_limit(&mut self, new: usize) -> usize {
        std::mem::replace(&mut self.size_limit, new)
    }

    /// Creates a new [`S2D`] with provided stream.
    pub fn new(inner: T, size_limit: usize) -> Self {
        Self { inner, size_limit }
    }
}
impl<T: AsyncRead + Unpin> S2D<T> {
    /// Receives a datagram from the stream.
    pub async fn recv(&mut self) -> anyhow::Result<Vec<u8>> {
        let len = self.inner.read_u64_le().await? as usize;
        if len > self.size_limit {
            return Err(anyhow!("datagram is too big ({} bytes)", len));
        }
        let mut blob = vec![0u8; len];
        self.inner.read_exact(&mut blob).await?;

        Ok(blob)
    }
}
impl<T: AsyncWrite + Unpin> S2D<T> {
    /// Sends a datagram to the stream.
    pub async fn send(&mut self, blob: &[u8]) -> anyhow::Result<()> {
        self.inner.write_u64_le(blob.len() as _).await?;
        self.inner.write_all(blob).await?;

        Ok(())
    }
}
impl<T: Read> S2D<T> {
    /// Receives a datagram from the stream.
    pub fn recv_blocking(&mut self) -> anyhow::Result<Vec<u8>> {
        let mut len = [0u8; std::mem::size_of::<u64>()];
        self.inner.read_exact(&mut len)?;
        let len = u64::from_le_bytes(len) as usize;
        if len > self.size_limit {
            return Err(anyhow!("datagram is too big ({} bytes)", len));
        }
        let mut blob = vec![0u8; len];
        self.inner.read_exact(&mut blob)?;

        Ok(blob)
    }
}
impl<T: Write> S2D<T> {
    /// Sends a datagram to the stream.
    pub fn send_blocking(&mut self, blob: &[u8]) -> anyhow::Result<()> {
        self.inner.write_all(&u64::to_le_bytes(blob.len() as _))?;
        self.inner.write_all(blob)?;

        Ok(())
    }
}
impl<T> AsRef<T> for S2D<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}
impl<T> AsMut<T> for S2D<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}
impl<T: AsyncRead + AsyncWrite + Unpin> From<T> for S2D<T> {
    fn from(inner: T) -> Self {
        Self::new(inner, 6 * 1024 * 1024)
    }
}
