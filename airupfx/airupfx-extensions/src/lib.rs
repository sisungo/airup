use airup_sdk::{
    info::ConnectionExt,
    ipc::MessageProto,
    nonblocking::ipc::{MessageProtoRecvExt, MessageProtoSendExt},
    system::{ConnectionExt as _, Event},
};
use airupfx_signal::SIGTERM;
use ciborium::cbor;
use std::{collections::HashMap, future::Future, path::Path, pin::Pin, sync::Arc};
use tokio::net::{
    unix::{OwnedReadHalf, OwnedWriteHalf},
    UnixListener,
};

#[derive(Debug)]
pub struct Server {
    listener: UnixListener,
    path: String,
    extension_name: String,
    service_name: String,
    rpc_methods: HashMap<String, Method>,
}
impl Server {
    pub async fn new(extension_name: impl Into<String>) -> anyhow::Result<Self> {
        let mut airup_rpc_conn =
            airup_sdk::nonblocking::Connection::connect(airup_sdk::socket_path()).await?;
        let build_manifest = airup_rpc_conn.build_manifest().await??;
        airup_sdk::build::try_set_manifest(build_manifest.clone());
        let service_name = std::env::var("AIRUP_SERVICE")?;
        let extension_socket_name = format!("airup_extension_{}.sock", service_name);
        let extension_socket_path = build_manifest
            .runtime_dir
            .join(&extension_socket_name)
            .display()
            .to_string();
        _ = std::fs::remove_file(&extension_socket_path);

        Ok(Self::with_config(extension_name, service_name, extension_socket_path).await?)
    }

    pub async fn with_config<P: AsRef<Path>>(
        extension_name: impl Into<String>,
        service_name: impl Into<String>,
        path: P,
    ) -> std::io::Result<Self> {
        let listener = UnixListener::bind(path.as_ref())?;
        airupfx_fs::set_permission(path.as_ref(), airupfx_fs::Permission::Socket).await?;

        Ok(Self {
            listener,
            path: path.as_ref().display().to_string(),
            extension_name: extension_name.into(),
            service_name: service_name.into(),
            rpc_methods: HashMap::with_capacity(16),
        })
    }

    pub fn mount(mut self, s: impl Into<String>, m: Method) -> Self {
        self.rpc_methods.insert(s.into(), m);
        self
    }

    pub async fn run(self) -> ! {
        let rpc_methods = Arc::new(self.rpc_methods);

        let extension_name = self.extension_name.clone();
        tokio::spawn(async move {
            let mut airup_rpc_conn =
                airup_sdk::nonblocking::Connection::connect(airup_sdk::socket_path()).await?;

            match airup_rpc_conn
                .load_extension(&extension_name, &self.path)
                .await
            {
                Ok(Ok(())) => (),
                Ok(Err(err)) => {
                    eprintln!("error: api failure: {err}");
                    std::process::exit(1);
                }
                Err(err) => {
                    eprintln!("error: rpc failure: {err}");
                    std::process::exit(1);
                }
            }

            _ = airup_rpc_conn
                .trigger_event(&Event::new("notify_active".into(), self.service_name))
                .await;

            Ok::<(), anyhow::Error>(())
        });

        let extension_name = self.extension_name.clone();
        _ = airupfx_signal::signal(SIGTERM, |_| async move {
            let Ok(mut airup_rpc_conn) =
                airup_sdk::nonblocking::Connection::connect(airup_sdk::socket_path()).await
            else {
                return;
            };

            _ = airup_rpc_conn.unload_extension(&extension_name).await;

            std::process::exit(0);
        });

        loop {
            let Ok((stream, _)) = self.listener.accept().await else {
                continue;
            };
            let (rx, tx) = stream.into_split();

            Session {
                rx: MessageProto::new(rx, 6 * 1024 * 1024),
                tx: Arc::new(MessageProto::new(tx, 6 * 1024 * 1024).into()),
                rpc_methods: rpc_methods.clone(),
            }
            .run_on_the_fly();
        }
    }
}

#[derive(Debug)]
struct Session {
    rx: MessageProto<OwnedReadHalf>,
    tx: Arc<tokio::sync::Mutex<MessageProto<OwnedWriteHalf>>>,
    rpc_methods: Arc<HashMap<String, Method>>,
}
impl Session {
    pub fn run_on_the_fly(self) {
        tokio::spawn(self.run());
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        loop {
            let request: airup_sdk::extension::Request =
                ciborium::from_reader(&self.rx.recv().await?[..])?;
            self.handle_request(request);
        }
    }

    fn handle_request(&self, request: airup_sdk::extension::Request) {
        let (tx, rpc_methods) = (self.tx.clone(), self.rpc_methods.clone());

        tokio::spawn(async move {
            let resp = match request.class {
                airup_sdk::extension::Request::CLASS_AIRUP_RPC => airup_sdk::extension::Response {
                    id: request.id,
                    data: ciborium::Value::serialized(
                        &Self::handle_rpc(rpc_methods, request.data.deserialized()?).await,
                    )?,
                },
                _ => airup_sdk::extension::Response {
                    id: request.id,
                    data: cbor!({})?,
                },
            };
            let mut buf = Vec::with_capacity(128);
            ciborium::into_writer(&resp, &mut buf)?;
            tx.lock().await.send(&buf).await?;

            Ok::<(), anyhow::Error>(())
        });
    }

    async fn handle_rpc(
        rpc_methods: Arc<HashMap<String, Method>>,
        request: airup_sdk::ipc::Request,
    ) -> airup_sdk::ipc::Response {
        match rpc_methods.get(&request.method) {
            Some(method) => airup_sdk::ipc::Response::new(method(request).await),
            None => airup_sdk::ipc::Response::new::<()>(Err(airup_sdk::Error::NoSuchMethod)),
        }
    }
}

pub type Method = fn(airup_sdk::ipc::Request) -> MethodFuture;

/// Represents to future type of an IPC method.
pub type MethodFuture =
    Pin<Box<dyn Future<Output = Result<ciborium::Value, airup_sdk::Error>> + Send>>;
