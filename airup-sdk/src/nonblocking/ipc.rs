use crate::{
    error::ApiError,
    ipc::{Error as IpcError, MessageProto, Request, Response},
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    future::Future,
    io::Cursor,
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

    /// Receives a datagram and deserializes it from CBOR to `T`.
    pub async fn recv<T: DeserializeOwned>(&mut self) -> Result<T, IpcError> {
        Ok(ciborium::from_reader(Cursor::new(self.0.recv().await?))?)
    }

    /// Receives a request from the underlying protocol.
    pub async fn recv_req(&mut self) -> Result<Request, IpcError> {
        let req: Request =
            ciborium::from_reader(Cursor::new(self.0.recv().await?)).unwrap_or_else(|err| {
                Request::new(
                    "debug.echo_raw",
                    Response::Err(ApiError::bad_request("InvalidJson", err.to_string())),
                )
            });
        Ok(req)
    }

    /// Receives a response from the underlying protocol.
    pub async fn recv_resp(&mut self) -> Result<Response, IpcError> {
        self.recv().await
    }

    /// Sends a datagram with CBOR-serialized given object.
    pub async fn send<T: Serialize>(&mut self, obj: &T) -> Result<(), IpcError> {
        let mut buffer = Cursor::new(Vec::with_capacity(128));
        ciborium::into_writer(obj, &mut buffer)?;
        self.0.send(&buffer.into_inner()).await
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

pub trait MessageProtoExt {
    /// Receives a message from the stream.
    fn recv(&mut self) -> impl Future<Output = Result<Vec<u8>, IpcError>>;

    /// Sends a message to the stream.
    fn send(&mut self, blob: &[u8]) -> impl Future<Output = Result<(), IpcError>>;
}
impl<T: AsyncRead + AsyncWrite + Unpin> MessageProtoExt for MessageProto<T> {
    async fn recv(&mut self) -> Result<Vec<u8>, IpcError> {
        let len = self.inner.read_u64_le().await? as usize;
        if len > self.size_limit {
            return Err(IpcError::MessageTooLong(len));
        }
        let mut blob = vec![0u8; len];
        self.inner.read_exact(&mut blob).await?;

        Ok(blob)
    }

    async fn send(&mut self, blob: &[u8]) -> Result<(), IpcError> {
        self.inner.write_u64_le(blob.len() as _).await?;
        self.inner.write_all(blob).await?;

        Ok(())
    }
}
