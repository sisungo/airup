pub mod files;
pub mod fs;
pub mod ipc;

use crate::{ipc::Request, Error};
use anyhow::anyhow;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    ops::{Deref, DerefMut},
    path::Path,
};

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
    pub fn send_raw(&mut self, msg: &[u8]) -> anyhow::Result<()> {
        (*self.underlying).send(msg)
    }

    /// Receives a raw message.
    pub fn recv_raw(&mut self) -> anyhow::Result<Vec<u8>> {
        (*self.underlying).recv()
    }

    /// Invokes an RPC method.
    pub fn invoke<P: Serialize, T: DeserializeOwned>(
        &mut self,
        method: &str,
        params: P,
    ) -> anyhow::Result<Result<T, Error>> {
        let req = Request::new(method, params).unwrap();
        self.underlying
            .send(&req)
            .map_err(|e| anyhow!("cannot send request to airup daemon: {e}"))?;
        Ok(self
            .underlying
            .recv_resp()
            .map_err(|e| anyhow!("cannot receive response from airup daemon: {e}"))?
            .into_result())
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
    type Invoke<'a, T: 'a> = anyhow::Result<Result<T, Error>>;

    fn invoke<'a, P: Serialize + 'a, T: DeserializeOwned + 'a>(
        &'a mut self,
        method: &'a str,
        params: P,
    ) -> Self::Invoke<'a, T> {
        self.invoke(method, params)
    }
}
