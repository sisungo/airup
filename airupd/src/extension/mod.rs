use airup_sdk::extension::{Request, Response};
use std::{
    collections::{HashMap, HashSet},
    process::Stdio,
    sync::{Arc, Mutex, RwLock},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{mpsc, oneshot},
};

use crate::app::airupd;

#[derive(Debug, Default)]
pub struct Extensions(RwLock<HashMap<String, Arc<Extension>>>);
impl Extensions {
    pub fn load(
        &self,
        name: String,
        cmdline: Vec<String>,
        methods: HashSet<String>,
    ) -> Result<(), airup_sdk::Error> {
        let mut lock = self.0.write().unwrap();
        if lock.contains_key(&name) {
            return Err(airup_sdk::Error::Exists);
        }
        lock.insert(
            name.clone(),
            Arc::new(
                Extension::new(name, cmdline, methods).map_err(|x| airup_sdk::Error::Io {
                    message: x.to_string(),
                })?,
            ),
        );
        Ok(())
    }

    pub async fn rpc_invoke(&self, req: airup_sdk::ipc::Request) -> airup_sdk::ipc::Response {
        let Some(ext) = self
            .0
            .read()
            .unwrap()
            .iter()
            .find(|(_, v)| v.methods.contains(&req.method))
            .map(|x| x.1)
            .cloned()
        else {
            return airup_sdk::ipc::Response::new::<()>(Err(airup_sdk::Error::NoSuchMethod));
        };

        ext.rpc_invoke(req).await.unwrap_or_else(|| {
            airup_sdk::ipc::Response::new::<()>(Err(airup_sdk::Error::Io {
                message: "extension communication error".into(),
            }))
        })
    }
}

#[derive(Debug)]
pub struct Extension {
    methods: HashSet<String>,
    gate: mpsc::Sender<(Request, oneshot::Sender<ciborium::Value>)>,
}
impl Extension {
    pub fn new(
        name: String,
        cmdline: Vec<String>,
        methods: HashSet<String>,
    ) -> std::io::Result<Self> {
        let cmd = cmdline.first().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "`cmdline` cannot be empty",
            )
        })?;
        let mut child = std::process::Command::new(cmd)
            .args(&cmdline[1..])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let stdout = child.stdout.take().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe not created")
        })?;
        let stdin = child.stdin.take().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe not created")
        })?;
        let (tx, rx) = mpsc::channel(8);
        ExtensionHost {
            name,
            stdin: tokio::process::ChildStdin::from_std(stdin)?,
            stdout: tokio::process::ChildStdout::from_std(stdout)?,
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
    stdin: tokio::process::ChildStdin,
    stdout: tokio::process::ChildStdout,
    reqs: HashMap<u64, oneshot::Sender<ciborium::Value>>,
    gate: mpsc::Receiver<(Request, oneshot::Sender<ciborium::Value>)>,
}
impl ExtensionHost {
    fn run_on_the_fly(mut self) {
        let reqs = Arc::new(Mutex::new(self.reqs));

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
                    let Ok(_) = self.stdin.write_u64_le(buf.len() as u64).await else {
                        return;
                    };
                    let Ok(_) = self.stdin.write_all(&buf).await else {
                        return;
                    };
                    let Ok(_) = self.stdin.flush().await else {
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
                    let Ok(len) = self.stdout.read_u64_le().await else {
                        return;
                    };
                    if len > 6 * 1024 * 1024 {
                        return;
                    }
                    let mut buf = vec![0u8; len as usize];
                    let Ok(_) = self.stdout.read_exact(&mut buf).await else {
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

            airupd().extensions.0.write().unwrap().remove(&self.name);
        });
    }
}
