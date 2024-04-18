use airup_sdk::{
    extension::{Request, Response},
    ipc::MessageProto,
    nonblocking::ipc::{MessageProtoRecvExt, MessageProtoSendExt},
};
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};
use tokio::{
    net::UnixStream,
    sync::{mpsc, oneshot},
};

use crate::app::airupd;

#[derive(Debug, Default)]
pub struct Extensions(tokio::sync::RwLock<HashMap<String, Arc<Extension>>>);
impl Extensions {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn load(
        &self,
        name: String,
        path: &str,
        methods: HashSet<String>,
    ) -> Result<(), airup_sdk::Error> {
        let mut lock = self.0.write().await;
        for (key, val) in lock.iter() {
            if key == &name || !val.methods.is_disjoint(&methods) {
                return Err(airup_sdk::Error::Exists);
            }
        }
        lock.insert(
            name.clone(),
            Arc::new(Extension::new(name, path, methods).await.map_err(|x| {
                airup_sdk::Error::Io {
                    message: x.to_string(),
                }
            })?),
        );
        Ok(())
    }

    pub async fn rpc_invoke(&self, mut req: airup_sdk::ipc::Request) -> airup_sdk::ipc::Response {
        let mut method_splited = req.method.splitn(2, '.');
        let extension = method_splited.next().unwrap();
        let Some(ext_method) = method_splited.next() else {
            return airup_sdk::ipc::Response::new::<()>(Err(airup_sdk::Error::NoSuchMethod));
        };
        let Some(ext) = self.0.read().await.get(extension).cloned() else {
            return airup_sdk::ipc::Response::new::<()>(Err(airup_sdk::Error::NoSuchMethod));
        };
        req.method = ext_method.into();

        ext.rpc_invoke(req).await.unwrap_or_else(|| {
            airup_sdk::ipc::Response::new::<()>(Err(airup_sdk::Error::Io {
                message: "extension communication error".into(),
            }))
        })
    }

    pub async fn unload(&self, name: &str) -> Result<(), airup_sdk::Error> {
        self.0
            .write()
            .await
            .remove(name)
            .ok_or(airup_sdk::Error::NotFound)
            .map(|_| ())
    }
}

#[derive(Debug)]
pub struct Extension {
    methods: HashSet<String>,
    gate: mpsc::Sender<(Request, oneshot::Sender<ciborium::Value>)>,
}
impl Extension {
    pub async fn new(name: String, path: &str, methods: HashSet<String>) -> std::io::Result<Self> {
        let (tx, rx) = mpsc::channel(8);
        let connection = UnixStream::connect(path).await?;
        ExtensionHost {
            name,
            connection,
            reqs: HashMap::with_capacity(8),
            gate: rx,
        }
        .run_on_the_fly();

        Ok(Self { methods, gate: tx })
    }

    pub async fn rpc_invoke(
        &self,
        req: airup_sdk::ipc::Request,
    ) -> Option<airup_sdk::ipc::Response> {
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

struct ExtensionHost {
    name: String,
    connection: UnixStream,
    reqs: HashMap<u64, oneshot::Sender<ciborium::Value>>,
    gate: mpsc::Receiver<(Request, oneshot::Sender<ciborium::Value>)>,
}
impl ExtensionHost {
    fn run_on_the_fly(mut self) {
        let reqs = Arc::new(Mutex::new(self.reqs));

        let (rx, tx) = self.connection.into_split();
        let mut rx = MessageProto::new(rx, 6 * 1024 * 1024);
        let mut tx = MessageProto::new(tx, 6 * 1024 * 1024);

        // accepting requests
        let mut acceptor = {
            let reqs = reqs.clone();
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
            let reqs = reqs.clone();
            tokio::spawn(async move {
                loop {
                    let Ok(buf) = rx.recv().await else {
                        return;
                    };
                    let Ok(resp) = ciborium::from_reader::<Response, _>(&buf[..]) else {
                        return;
                    };
                    let Some(req_chan) = reqs.lock().unwrap().remove(&resp.id) else {
                        return;
                    };
                    req_chan.send(resp.data).ok();
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

            airupd().extensions.0.write().await.remove(&self.name);
        });
    }
}
