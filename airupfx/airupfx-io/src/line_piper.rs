//! A cancel-safe solution for reading lines from a stream.

use std::{future::Future, pin::Pin};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    sync::mpsc,
    task::{AbortHandle, JoinHandle},
};

/// Asynchronous callback used for line pipers.
///
/// We chose to create our own callback trait, instead of using standard [`FnOnce`], because several lifetime issues around
/// closure types has not been stablized yet.
///
/// The callback should be cancel-safe, since the task is cancelled when [`CallbackGuard`] is dropped.
pub trait Callback: Send + Sync {
    fn invoke<'a>(&'a self, a: &'a [u8]) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>>;
    fn clone_boxed(&self) -> Box<dyn Callback>;
}

#[derive(Debug)]
pub struct LinePiper {
    rx: tokio::sync::Mutex<mpsc::Receiver<Vec<u8>>>,
    _guard: CallbackGuard,
}
impl LinePiper {
    pub fn new(reader: impl AsyncRead + Unpin + Send + 'static) -> Self {
        let (tx, rx) = mpsc::channel(4);
        let _guard = set_callback(reader, Box::new(ChannelCallback::new(tx)));
        Self {
            rx: rx.into(),
            _guard,
        }
    }

    pub async fn read_line(&self) -> Option<Vec<u8>> {
        self.rx.lock().await.recv().await
    }
}

#[must_use]
#[derive(Debug)]
pub struct CallbackGuard(AbortHandle);
impl Drop for CallbackGuard {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Sets up a line piper and sets a callback for it. The line piper is automatically closed when stream `reader` reached EOF.
pub fn set_callback(
    reader: impl AsyncRead + Unpin + Send + 'static,
    callback: Box<dyn Callback>,
) -> CallbackGuard {
    CallbackGuard(
        LinePiperEntity::new(reader, callback)
            .start()
            .abort_handle(),
    )
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
    use std::sync::{Arc, Mutex};

    const TEST_STRING: &'static [u8] = b"AirupFX Test\nairupfx::io::line_piper\n";

    #[tokio::test]
    async fn multi_line() {
        let line_piper = LinePiper::new(TEST_STRING);
        assert_eq!(line_piper.read_line().await.unwrap(), TEST_STRING);
    }

    #[tokio::test]
    async fn callback() {
        #[derive(Clone)]
        struct TestCallback(Arc<Mutex<Vec<u8>>>);
        impl Callback for TestCallback {
            fn clone_boxed(&self) -> Box<dyn Callback> {
                Box::new(self.clone())
            }

            fn invoke<'a>(&'a self, a: &'a [u8]) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
                Box::pin(async {
                    self.0.lock().unwrap().append(&mut a.to_vec());
                })
            }
        }

        let buffer = Default::default();
        let callback = TestCallback(Arc::clone(&buffer));
        let _guard = set_callback(TEST_STRING, Box::new(callback));
        while Arc::strong_count(&buffer) > 1 {
            tokio::task::yield_now().await;
        }
        assert_eq!(*buffer.lock().unwrap(), TEST_STRING);
    }
}
