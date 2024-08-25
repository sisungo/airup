//! A cancel-safe solution for reading lines from a stream.

use std::{future::Future, pin::Pin};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    sync::mpsc,
    task::JoinHandle,
};

/// Asynchronous callback used for line pipers.
///
/// We chose to create our own callback trait, instead of using standard [FnOnce], because several lifetime issues around
/// closure types has not been stablized yet.
pub trait Callback: Send + Sync {
    fn invoke<'a>(&'a self, a: &'a [u8]) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
    fn clone_boxed(&self) -> Box<dyn Callback>;
}

#[derive(Debug)]
pub struct LinePiper {
    rx: tokio::sync::Mutex<mpsc::Receiver<Vec<u8>>>,
    join_handle: JoinHandle<()>,
}
impl LinePiper {
    pub fn new(reader: impl AsyncRead + Unpin + Send + 'static) -> Self {
        let (tx, rx) = mpsc::channel(4);
        let join_handle = LinePiperEntity::new(reader, Box::new(ChannelCallback::new(tx))).start();
        Self {
            rx: rx.into(),
            join_handle,
        }
    }

    pub async fn read_line(&self) -> Option<Vec<u8>> {
        self.rx.lock().await.recv().await
    }
}
impl Drop for LinePiper {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

/// Sets up a line piper and sets a callback for it. The line piper is automatically closed when stream `reader` reached EOF.
pub fn set_callback(
    reader: impl AsyncRead + Unpin + Send + 'static,
    callback: Box<dyn Callback>,
) -> JoinHandle<()> {
    LinePiperEntity::new(reader, callback).start()
}

struct LinePiperEntity<R> {
    reader: R,
    callback: Box<dyn Callback>,
}
impl<R: AsyncRead + Unpin + Send + 'static> LinePiperEntity<R> {
    const LINE_BREAK_BYTES: &'static [u8] = b"\n\r";

    fn new(reader: R, callback: Box<dyn Callback>) -> Self {
        Self { reader, callback }
    }

    fn start(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            self.run().await;
        })
    }

    async fn run(mut self) -> Option<()> {
        let mut buf = Box::new([0u8; 4096]);
        let mut position = 0;
        loop {
            let pos = loop {
                let count = self.reader.read(&mut buf[position..]).await.ok()?;
                if count == 0 {
                    return None;
                }
                position += count;
                if let Some(pos) = &buf[..position]
                    .iter()
                    .rposition(|x| Self::LINE_BREAK_BYTES.contains(x))
                {
                    break pos + 1;
                }
                assert!(position <= 4096);
                if position == 4096 {
                    break 4096;
                }
            };
            self.callback.invoke(&buf[..pos]).await;
            position = 0;
        }
    }
}

#[derive(Clone)]
pub struct ChannelCallback {
    tx: mpsc::Sender<Vec<u8>>,
}
impl ChannelCallback {
    pub fn new(tx: mpsc::Sender<Vec<u8>>) -> Self {
        Self { tx }
    }
}
impl Callback for ChannelCallback {
    fn clone_boxed(&self) -> Box<dyn Callback> {
        Box::new(self.clone())
    }

    fn invoke<'a>(&'a self, a: &'a [u8]) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async {
            _ = self.tx.send(a.to_vec()).await;
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn multi_line() {
        const TEST_STRING: &'static [u8] = b"AirupFX Test\nairupfx::io::line_piper\n";

        let line_piper = LinePiper::new(TEST_STRING);
        assert_eq!(line_piper.read_line().await.unwrap(), TEST_STRING);
    }
}
