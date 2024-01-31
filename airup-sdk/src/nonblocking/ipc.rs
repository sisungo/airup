use crate::{
    error::ApiError,
    ipc::{Error as IpcError, Request, Response, DEFAULT_SIZE_LIMIT},
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    ops::{Deref, DerefMut},
    path::Path,
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{UnixListener, UnixStream},
};

#[derive(Debug)]
pub struct Connection(MessageProto<UnixStream>);
impl Connection {
    /// Connects to the specified socket.
    pub async fn connect<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        Ok(Self(MessageProto::new(
            UnixStream::connect(path).await?,
            usize::MAX,
        )))
    }

    /// Receives a datagram and deserializes it from JSON to `T`.
    pub async fn recv<T: DeserializeOwned>(&mut self) -> Result<T, IpcError> {
        Ok(serde_json::from_slice(&self.0.recv().await?)?)
    }

    /// Receives a request from the underlying protocol.
    pub async fn recv_req(&mut self) -> Result<Request, IpcError> {
        let req: Request = serde_json::from_slice(&self.0.recv().await?).unwrap_or_else(|err| {
            Request::new(
                "debug.echo_raw",
                Response::Err(ApiError::bad_request("InvalidJson", err.to_string())),
            )
            .unwrap()
        });
        Ok(req)
    }

    /// Receives a response from the underlying protocol.
    pub async fn recv_resp(&mut self) -> Result<Response, IpcError> {
        self.recv().await
    }

    /// Sends a datagram with JSON-serialized given object.
    pub async fn send<T: Serialize>(&mut self, obj: &T) -> Result<(), IpcError> {
        self.0.send(serde_json::to_string(obj)?.as_bytes()).await
    }
}
impl Deref for Connection {
    type Target = MessageProto<UnixStream>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Connection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A wrap of `UnixListener` that accepts [`Connection`].
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

/// A middle layer that splits a stream into messages.
#[derive(Debug)]
pub struct MessageProto<T> {
    inner: T,
    size_limit: usize,
}
impl<T> MessageProto<T> {
    /// Sets received datagram size limitation.
    pub fn set_size_limit(&mut self, new: usize) -> usize {
        std::mem::replace(&mut self.size_limit, new)
    }

    /// Creates a new [`MessageProto`] with provided stream.
    pub fn new(inner: T, size_limit: usize) -> Self {
        Self { inner, size_limit }
    }
}
impl<T: AsyncRead + Unpin> MessageProto<T> {
    /// Receives a datagram from the stream.
    pub async fn recv(&mut self) -> Result<Vec<u8>, IpcError> {
        let len = self.inner.read_u64_le().await? as usize;
        if len > self.size_limit {
            return Err(IpcError::MessageTooLong);
        }
        let mut blob = vec![0u8; len];
        self.inner.read_exact(&mut blob).await?;

        Ok(blob)
    }
}
impl<T: AsyncWrite + Unpin> MessageProto<T> {
    /// Sends a datagram to the stream.
    pub async fn send(&mut self, blob: &[u8]) -> Result<(), IpcError> {
        self.inner.write_u64_le(blob.len() as _).await?;
        self.inner.write_all(blob).await?;

        Ok(())
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
impl<T: AsyncRead + AsyncWrite + Unpin> From<T> for MessageProto<T> {
    fn from(inner: T) -> Self {
        Self::new(inner, DEFAULT_SIZE_LIMIT)
    }
}
