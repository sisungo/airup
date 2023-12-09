use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader},
    sync::mpsc,
    task::JoinHandle,
};

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

    pub async fn read_line(&self) -> Option<Vec<u8>> {
        self.rx.lock().await.recv().await
    }
}
impl Drop for PiperHandle {
    fn drop(&mut self) {
        self.join_handle.abort();
    }
}

#[derive(Debug)]
struct Piper<R> {
    reader: R,
    tx: mpsc::Sender<Vec<u8>>,
}
impl<R: AsyncRead + Unpin + Send + 'static> Piper<R> {
    fn new(reader: R, tx: mpsc::Sender<Vec<u8>>) -> Self {
        Self { reader, tx }
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
            self.tx.send(buf.clone()).await.ok()?;
            buf.clear();
        }
    }
}
