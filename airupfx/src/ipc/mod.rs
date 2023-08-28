//! # Airup IPC

pub mod gram;
pub mod mapi;

use self::mapi::{ApiError, Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    ops::{Deref, DerefMut},
    path::Path,
};
use tokio::net::{unix::UCred, UnixListener, UnixStream};

/// Represents to a connection.
#[derive(Debug)]
pub struct Connection(gram::Connection<UnixStream>);
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
    type Target = gram::Connection<UnixStream>;

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
