pub mod files;
pub mod fs;
pub mod rpc;

use crate::{
    error::ApiError,
    rpc::{Error as IpcError, Request},
};
use rpc::{MessageProtoRecvExt, MessageProtoSendExt};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

/// A high-level wrapper of a connection to `airupd`.
#[derive(Debug)]
pub struct Connection {
    underlying: rpc::Connection,
}
impl Connection {
    /// Connects to the specific path.
    pub fn connect<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        Ok(Self {
            underlying: rpc::Connection::connect(path)?,
        })
    }

    /// Sends a raw message.
    pub fn send_raw(&mut self, msg: &[u8]) -> Result<(), IpcError> {
        (*self.underlying).send(msg)
    }

    /// Receives a raw message.
    pub fn recv_raw(&mut self) -> Result<Vec<u8>, IpcError> {
        let mut buf = Vec::new();
        (*self.underlying).recv(&mut buf)?;
        Ok(buf)
    }

    /// Invokes an RPC method.
    pub fn invoke<P: Serialize, T: DeserializeOwned>(
        &mut self,
        method: &str,
        params: P,
    ) -> Result<Result<T, ApiError>, IpcError> {
        let req = Request::new(method, params);
        self.underlying.send(&req)?;
        Ok(self
            .underlying
            .recv::<crate::rpc::Response>()?
            .into_result())
    }

    pub fn into_inner(self) -> rpc::Connection {
        self.underlying
    }
}
impl Deref for Connection {
    type Target = rpc::Connection;

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
