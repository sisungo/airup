pub mod files;
pub mod fs;
pub mod ipc;

use crate::{
    error::ApiError,
    ipc::{Error as IpcError, Request},
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    ops::{Deref, DerefMut},
    path::Path,
};
use ipc::MessageProtoExt;

/// A high-level wrapper of a connection to `airupd`.
#[derive(Debug)]
pub struct Connection {
    underlying: ipc::Connection,
}
impl Connection {
    /// Connects to the specific path.
    pub fn connect<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        Ok(Self {
            underlying: ipc::Connection::connect(path)?,
        })
    }

    /// Sends a raw message.
    pub fn send_raw(&mut self, msg: &[u8]) -> Result<(), IpcError> {
        (*self.underlying).send(msg)
    }

    /// Receives a raw message.
    pub fn recv_raw(&mut self) -> Result<Vec<u8>, IpcError> {
        (*self.underlying).recv()
    }

    /// Invokes an RPC method.
    pub fn invoke<P: Serialize, T: DeserializeOwned>(
        &mut self,
        method: &str,
        params: P,
    ) -> Result<Result<T, ApiError>, IpcError> {
        let req = Request::new(method, params).unwrap();
        self.underlying.send(&req)?;
        Ok(self.underlying.recv_resp()?.into_result())
    }
}
impl Deref for Connection {
    type Target = ipc::Connection;

    fn deref(&self) -> &Self::Target {
        &self.underlying
    }
}
impl DerefMut for Connection {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.underlying
    }
}

impl crate::Connection for Connection {
    type Invoke<'a, T: 'a> = Result<Result<T, ApiError>, IpcError>;

    fn invoke<'a, P: Serialize + 'a, T: DeserializeOwned + 'a>(
        &'a mut self,
        method: &'a str,
        params: P,
    ) -> Self::Invoke<'a, T> {
        self.invoke(method, params)
    }
}
