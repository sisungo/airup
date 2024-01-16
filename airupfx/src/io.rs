//! Utility for I/O.

use std::{future::Future, pin::Pin};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader},
    sync::mpsc,
    task::JoinHandle,
};

pub trait PiperCallback:
    FnOnce(Vec<u8>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync
{
    fn clone_boxed(&self) -> Box<dyn PiperCallback>;
}
impl<T> PiperCallback for T
where
    T: FnOnce(Vec<u8>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Clone + Send + Sync + 'static,
{
    fn clone_boxed(&self) -> Box<dyn PiperCallback> {
        Box::new(self.clone())
    }
}

#[derive(Debug)]
pub struct PiperHandle {
    rx: tokio::sync::Mutex<mpsc::Receiver<Vec<u8>>>,
    join_handle: JoinHandle<()>,
}
impl PiperHandle {
    pub fn new(reader: impl AsyncRead + Unpin + Send + 'static) -> Self {
        let (tx, rx) = mpsc::channel(4);
        let join_handle = Piper::new(reader, tx).start();
        Self {
            rx: rx.into(),
            join_handle,
        }
    }

    pub fn with_callback(
        reader: impl AsyncRead + Unpin + Send + 'static,
        callback: Box<dyn PiperCallback>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(4);
        let join_handle: JoinHandle<()> = Piper::with_callback(reader, tx, callback).start();
        Self {
            rx: rx.into(),
            join_handle,
        }
    }

    pub async fn read_line(&self) -> Option<Vec<u8>> {
        self.rx.lock().await.recv().await
    }
}
impl Drop for PiperHandle {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

struct Piper<R> {
    reader: R,
    tx: mpsc::Sender<Vec<u8>>,
    callback: Option<Box<dyn PiperCallback>>,
}
impl<R: AsyncRead + Unpin + Send + 'static> Piper<R> {
    fn new(reader: R, tx: mpsc::Sender<Vec<u8>>) -> Self {
        Self {
            reader,
            tx,
            callback: None,
        }
    }

    fn with_callback(
        reader: R,
        tx: mpsc::Sender<Vec<u8>>,
        callback: Box<dyn PiperCallback>,
    ) -> Self {
        Self {
            reader,
            tx,
            callback: Some(callback),
        }
    }

    fn start(self) -> JoinHandle<()> {
        tokio::spawn(async move {
            self.run().await;
        })
    }

    async fn run(self) -> Option<()> {
        let mut buf = Vec::new();
        let mut buf_reader = BufReader::new(self.reader);
        loop {
            let mut limited = (&mut buf_reader).take(1024 * 4);
            limited.read_until(b'\n', &mut buf).await.ok()?;
            match &self.callback {
                Some(callback) => {
                    (callback.clone_boxed())(buf.clone()).await;
                }
                None => {
                    self.tx.send(buf.clone()).await.ok()?;
                }
            }
            buf.clear();
        }
    }
}
