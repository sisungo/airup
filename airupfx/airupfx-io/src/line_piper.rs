use std::{future::Future, pin::Pin};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader},
    sync::mpsc,
    task::JoinHandle,
};

pub trait LinePiperCallback:
    FnOnce(Vec<u8>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync
{
    fn clone_boxed(&self) -> Box<dyn LinePiperCallback>;
}
impl<T> LinePiperCallback for T
where
    T: FnOnce(Vec<u8>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Clone + Send + Sync + 'static,
{
    fn clone_boxed(&self) -> Box<dyn LinePiperCallback> {
        Box::new(self.clone())
    }
}

#[derive(Debug)]
pub struct LinePiper {
    rx: tokio::sync::Mutex<mpsc::Receiver<Vec<u8>>>,
    join_handle: JoinHandle<()>,
}
impl LinePiper {
    pub fn new(reader: impl AsyncRead + Unpin + Send + 'static) -> Self {
        let (tx, rx) = mpsc::channel(4);
        let join_handle = LinePiperEntity::new(reader, tx).start();
        Self {
            rx: rx.into(),
            join_handle,
        }
    }

    pub fn with_callback(
        reader: impl AsyncRead + Unpin + Send + 'static,
        callback: Box<dyn LinePiperCallback>,
    ) -> Self {
        let (tx, rx) = mpsc::channel(4);
        let join_handle: JoinHandle<()> =
            LinePiperEntity::with_callback(reader, tx, callback).start();
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

struct LinePiperEntity<R> {
    reader: R,
    tx: mpsc::Sender<Vec<u8>>,
    callback: Option<Box<dyn LinePiperCallback>>,
}
impl<R: AsyncRead + Unpin + Send + 'static> LinePiperEntity<R> {
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
        callback: Box<dyn LinePiperCallback>,
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
