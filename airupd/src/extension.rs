use crate::app::airupd;
use airup_sdk::{
    extension::{Request, Response},
    nonblocking::rpc::{MessageProtoRecvExt, MessageProtoSendExt},
    rpc::MessageProto,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::{
    net::UnixStream,
    sync::{mpsc, oneshot},
    task::JoinHandle,
};

/// Represents to an extension manager.
#[derive(Debug, Default)]
pub struct Extensions(std::sync::RwLock<HashMap<String, Arc<Extension>>>);
impl Extensions {
    /// Creates a new [`Extensions`] instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers an extension.
    pub fn register(&self, name: String, conn: UnixStream) -> Result<(), airup_sdk::Error> {
        let mut lock = self.0.write().unwrap();
        if lock.contains_key(&name) {
            return Err(airup_sdk::Error::Exists);
        }
        lock.insert(
            name.clone(),
            Arc::new(
                Extension::new(name, conn).map_err(|x| airup_sdk::Error::Io {
                    message: x.to_string(),
                })?,
            ),
        );
        Ok(())
    }

    /// Invokes an RPC invokation on an extension.
    pub fn rpc_invoke(
        &self,
        mut req: airup_sdk::rpc::Request,
    ) -> JoinHandle<airup_sdk::rpc::Response> {
        let mut method_splited = req.method.splitn(2, '.');
        let extension = method_splited.next().unwrap();
        let Some(ext_method) = method_splited.next() else {
            return tokio::spawn(std::future::ready(airup_sdk::rpc::Response::new::<()>(
                Err(airup_sdk::Error::NotImplemented),
            )));
        };
        let Some(ext) = self.0.read().unwrap().get(extension).cloned() else {
            return tokio::spawn(std::future::ready(airup_sdk::rpc::Response::new::<()>(
                Err(airup_sdk::Error::NotImplemented),
            )));
        };
        req.method = ext_method.into();

        tokio::spawn(async move {
            ext.rpc_invoke(req).await.unwrap_or_else(|| {
                airup_sdk::rpc::Response::new::<()>(Err(airup_sdk::Error::Io {
                    message: "extension communication error".into(),
                }))
            })
        })
    }

    /// Unregisters an extension.
    pub fn unregister(&self, name: &str) -> Result<(), airup_sdk::Error> {
        self.0
            .write()
            .unwrap()
            .remove(name)
            .ok_or(airup_sdk::Error::NotFound)
            .map(|_| ())
    }
}

/// Interface to a hosting extension.
#[derive(Debug)]
struct Extension {
    gate: mpsc::Sender<(Request, oneshot::Sender<ciborium::Value>)>,
}
impl Extension {
    /// Creates a new [`Extension`] instance, hosting the extension.
    fn new(name: String, connection: UnixStream) -> std::io::Result<Self> {
        let (tx, rx) = mpsc::channel(8);
        ExtensionHost {
            name,
            connection,
            reqs: HashMap::with_capacity(8),
            gate: rx,
        }
        .run_on_the_fly();

        Ok(Self { gate: tx })
    }

    /// Invokes an RPC invokation on the extension.
    async fn rpc_invoke(&self, req: airup_sdk::rpc::Request) -> Option<airup_sdk::rpc::Response> {
        let req = Request {
            id: 0,
            class: Request::CLASS_AIRUP_RPC,
            data: ciborium::Value::serialized(&req).unwrap(),
        };
        let (tx, rx) = oneshot::channel();
        self.gate.send((req, tx)).await.ok()?;
        rx.await.ok().and_then(|x| x.deserialized().ok())
    }
}

/// Background task for holding an extension.
struct ExtensionHost {
    name: String,
    connection: UnixStream,
    reqs: HashMap<u64, oneshot::Sender<ciborium::Value>>,
    gate: mpsc::Receiver<(Request, oneshot::Sender<ciborium::Value>)>,
}
impl ExtensionHost {
    /// Maximum size of received message from an extension, in bytes.
    const SIZE_LIMIT: usize = 8 * 1024 * 1024;

    /// Runs the host on the fly.
    fn run_on_the_fly(mut self) {
        // FIXME: Using `HashMap` here may be slow and memory-consuming.
        let reqs = Arc::new(Mutex::new(self.reqs));

        let (rx, tx) = self.connection.into_split();
        let mut rx = MessageProto::new(rx, Self::SIZE_LIMIT);
        let mut tx = MessageProto::new(tx, Self::SIZE_LIMIT);

        // accepting requests
        let mut acceptor = {
            let reqs = Arc::clone(&reqs);
            tokio::spawn(async move {
                let mut req_id = 1;
                while let Some((mut req, sender)) = self.gate.recv().await {
                    req.id = req_id;
                    reqs.lock().unwrap().insert(req_id, sender);
                    req_id += 1;
                    let mut buf = Vec::with_capacity(128);
                    ciborium::into_writer(&req, &mut buf)
                        .expect("writing to `Vec<u8>` should never fail");
                    if tx.send(&buf).await.is_err() {
                        return;
                    };
                }
            })
        };

        // handling responses
        let mut handler = {
            let reqs = Arc::clone(&reqs);
            tokio::spawn(async move {
                let mut buf = Vec::with_capacity(4096);
                loop {
                    if rx.recv(&mut buf).await.is_err() {
                        return;
                    };
                    let Ok(resp) = ciborium::from_reader::<Response, _>(&buf[..]) else {
                        return;
                    };
                    let Some(req_chan) = reqs.lock().unwrap().remove(&resp.id) else {
                        return;
                    };
                    _ = req_chan.send(resp.data);
                }
            })
        };

        // supervising them
        tokio::spawn(async move {
            tokio::select! {
                _ = &mut acceptor => {
                    handler.abort();
                },
                _ = &mut handler => {
                    acceptor.abort();
                },
            };

            airupd().extensions.0.write().unwrap().remove(&self.name);
        });
    }
}
