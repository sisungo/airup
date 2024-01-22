pub mod files;
pub mod fs;
pub mod ipc;

use crate::{ipc::Request, Error};
use anyhow::anyhow;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    future::Future,
    ops::{Deref, DerefMut},
    path::Path,
    pin::Pin,
};

/// A high-level wrapper of a connection to `airupd`.
#[derive(Debug)]
pub struct Connection {
    underlying: ipc::Connection,
}
impl Connection {
    /// Connects to the specific path.
    pub async fn connect<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        Ok(Self {
            underlying: ipc::Connection::connect(path).await?,
        })
    }

    /// Sends a raw message.
    pub async fn send_raw(&mut self, msg: &[u8]) -> anyhow::Result<()> {
        (*self.underlying).send(msg).await
    }

    /// Receives a raw message.
    pub async fn recv_raw(&mut self) -> anyhow::Result<Vec<u8>> {
        (*self.underlying).recv().await
    }

    /// Invokes an RPC method.
    pub async fn invoke<P: Serialize, T: DeserializeOwned>(
        &mut self,
        method: &str,
        params: P,
    ) -> anyhow::Result<Result<T, Error>> {
        let req = Request::new(method, params).unwrap();
        self.underlying
            .send(&req)
            .await
            .map_err(|e| anyhow!("cannot send request to airup daemon: {e}"))?;
        Ok(self
            .underlying
            .recv_resp()
            .await
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
    type Invoke<'a, T: 'a> = Pin<Box<dyn Future<Output = anyhow::Result<Result<T, Error>>> + 'a>>;

    fn invoke<'a, P: Serialize + 'a, T: DeserializeOwned + 'a>(
        &'a mut self,
        method: &'a str,
        params: P,
    ) -> Self::Invoke<'a, T> {
        Box::pin(self.invoke(method, params))
    }
}
