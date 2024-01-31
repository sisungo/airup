use crate::{
    error::ApiError,
    ipc::{Error as IpcError, Request, Response, DEFAULT_SIZE_LIMIT},
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    io::{Read, Write},
    ops::{Deref, DerefMut},
    os::unix::net::UnixStream,
    path::Path,
};

#[derive(Debug)]
pub struct Connection(MessageProto<UnixStream>);
impl Connection {
    /// Connects to the specified socket.
    pub fn connect<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        Ok(Self(MessageProto::new(
            UnixStream::connect(path)?,
            usize::MAX,
        )))
    }

    /// Receives a datagram and deserializes it from JSON to `T`.
    pub fn recv<T: DeserializeOwned>(&mut self) -> Result<T, IpcError> {
        Ok(serde_json::from_slice(&self.0.recv()?)?)
    }

    /// Receives a request from the underlying protocol.
    pub fn recv_req(&mut self) -> Result<Request, IpcError> {
        let req: Request = serde_json::from_slice(&self.0.recv()?).unwrap_or_else(|err| {
            Request::new(
                "debug.echo_raw",
                Response::Err(ApiError::bad_request("InvalidJson", err.to_string())),
            )
            .unwrap()
        });
        Ok(req)
    }

    /// Receives a response from the underlying protocol.
    pub fn recv_resp(&mut self) -> Result<Response, IpcError> {
        self.recv()
    }

    /// Sends a datagram with JSON-serialized given object.
    pub fn send<T: Serialize>(&mut self, obj: &T) -> Result<(), IpcError> {
        self.0.send(serde_json::to_string(obj)?.as_bytes())
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
impl<T: Read> MessageProto<T> {
    /// Receives a datagram from the stream.
    pub fn recv(&mut self) -> Result<Vec<u8>, IpcError> {
        let mut len = [0u8; 8];
        self.inner.read_exact(&mut len)?;
        let len = u64::from_le_bytes(len) as usize;
        if len > self.size_limit {
            return Err(IpcError::MessageTooLong);
        }
        let mut blob = vec![0u8; len];
        self.inner.read_exact(&mut blob)?;

        Ok(blob)
    }
}
impl<T: Write> MessageProto<T> {
    /// Sends a datagram to the stream.
    pub fn send(&mut self, blob: &[u8]) -> Result<(), IpcError> {
        self.inner.write_all(&u64::to_le_bytes(blob.len() as _))?;
        self.inner.write_all(blob)?;

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
impl<T: Read + Write> From<T> for MessageProto<T> {
    fn from(inner: T) -> Self {
        Self::new(inner, DEFAULT_SIZE_LIMIT)
    }
}
