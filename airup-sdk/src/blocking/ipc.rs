use crate::{
    error::ApiError,
    ipc::{Error as IpcError, Request, Response, MessageProto},
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

pub trait MessageProtoExt {
    /// Receives a message from the stream.
    fn recv(&mut self) -> Result<Vec<u8>, IpcError>;

    /// Sends a message to the stream
    fn send(&mut self, blob: &[u8]) -> Result<(), IpcError>;
}
impl<T: Read + Write> MessageProtoExt for MessageProto<T> {
    fn recv(&mut self) -> Result<Vec<u8>, IpcError> {
        let mut len = [0u8; 8];
        self.inner.read_exact(&mut len)?;
        let len = u64::from_le_bytes(len) as usize;
        if len > self.size_limit {
            return Err(IpcError::MessageTooLong(len));
        }
        let mut blob = vec![0u8; len];
        self.inner.read_exact(&mut blob)?;

        Ok(blob)
    }

    fn send(&mut self, blob: &[u8]) -> Result<(), IpcError> {
        self.inner.write_all(&u64::to_le_bytes(blob.len() as _))?;
        self.inner.write_all(blob)?;

        Ok(())
    }
}
