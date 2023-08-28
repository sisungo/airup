//! # Airup IPC Protocol - Stream to Datagram
//! Airup uses a very simple protocol to wrap streaming protocols to datagram protocols.
//!
//! ## The Conversation Model
//! The simple protocol is a half-duplex, synchronous "blocking" protocol. The client sends a request and the server sends a
//! response. If an serious protocol error occured, the connection will be simply shut down.
//!
//! ## Datagram Layout
//! A datagram begins with a 64-bit long, little-endian ordered integer, which represents to the datagram's length. The length
//! should be less than 6MiB and cannot be zero, or a serious protocol error will be occured. Then follows content of the
//! datagram.

use anyhow::anyhow;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// Represents to a connection.
#[derive(Debug)]
pub struct Connection<T>(T);
impl<T> Connection<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    /// Maximum length of a datagram.
    pub const MAX_DATAGRAM_LEN: usize = 6 * 1024 * 1024;

    /// Receives a datagram.
    pub async fn recv(&mut self) -> anyhow::Result<Vec<u8>> {
        let len = self.0.read_u64_le().await? as usize;
        if len > Self::MAX_DATAGRAM_LEN {
            return Err(anyhow!("datagram is too big ({} bytes)", len));
        }
        let mut blob = vec![0u8; len];
        self.0.read_exact(&mut blob).await?;

        Ok(blob)
    }

    /// Sends a datagram.
    pub async fn send(&mut self, blob: &[u8]) -> anyhow::Result<()> {
        if blob.len() > Self::MAX_DATAGRAM_LEN {
            return Err(anyhow!("datagram is too big ({} bytes)", blob.len()));
        }
        self.0.write_u64_le(blob.len() as _).await?;
        self.0.write_all(blob).await?;

        Ok(())
    }

    /// Creates a new `Connection` with provided stream.
    pub fn new(stream: T) -> Self {
        stream.into()
    }
}
impl<T> AsRef<T> for Connection<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
impl<T> AsMut<T> for Connection<T> {
    fn as_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
impl<T> From<T> for Connection<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}
