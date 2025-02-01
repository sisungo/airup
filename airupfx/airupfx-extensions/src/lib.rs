//! # AirupFX Extension Framework
//! This crate provides a high-level framework for writing Airup extensions in async Rust easily.

use airup_sdk::{
    info::ConnectionExt,
    nonblocking::rpc::{MessageProtoRecvExt, MessageProtoSendExt},
    rpc::{MessageProto, Request},
    system::{ConnectionExt as _, Event},
};
use airupfx_signal::SIGTERM;
use ciborium::cbor;
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};
use tokio::net::unix::{OwnedReadHalf, OwnedWriteHalf};

/// An extension server.
#[derive(Debug)]
pub struct Server {
    extension_name: String,
    service_name: String,
    rpc_methods: HashMap<&'static str, Method>,
}
impl Server {
    /// Creates a new [`Server`] instance, which is going to register to the Airup daemon with given extension name.
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

        Ok(Self::with_config(extension_name, service_name).await?)
    }

    /// Creates a new [`Server`] instance with custom config, instead of fetching from the environment.
    pub async fn with_config(
        extension_name: impl Into<String>,
        service_name: impl Into<String>,
    ) -> std::io::Result<Self> {
        Ok(Self {
            extension_name: extension_name.into(),
            service_name: service_name.into(),
            rpc_methods: HashMap::with_capacity(16),
        })
    }

    /// Mounts specific RPC method to specified handler.
    pub fn mount(mut self, name: &'static str, handler: Method) -> Self {
        self.rpc_methods.insert(name, handler);
        self
    }

    /// Runs the extension server.
    pub async fn run(self) -> anyhow::Result<()> {
        let rpc_methods = Arc::new(self.rpc_methods);

        let extension_name = self.extension_name.clone();
        let mut extension_conn =
            airup_sdk::nonblocking::Connection::connect(airup_sdk::socket_path()).await?;

        extension_conn
            .send(&Request::new("session.into_extension", extension_name))
            .await?;

        _ = airup_sdk::nonblocking::Connection::connect(airup_sdk::socket_path())
            .await?
            .trigger_event(&Event::new("notify_active".into(), self.service_name))
            .await;

        let extension_name = self.extension_name.clone();
        _ = airupfx_signal::signal(SIGTERM, |_| async move {
            let Ok(mut airup_rpc_conn) =
                airup_sdk::nonblocking::Connection::connect(airup_sdk::socket_path()).await
            else {
                return;
            };

            _ = airup_rpc_conn.unregister_extension(&extension_name).await;

            std::process::exit(0);
        });

        let stream = extension_conn.into_inner().into_inner().into_inner();
        let (rx, tx) = stream.into_split();

        ServerImpl {
            rx: MessageProto::new(rx, 6 * 1024 * 1024),
            tx: Arc::new(MessageProto::new(tx, 6 * 1024 * 1024).into()),
            rpc_methods: rpc_methods.clone(),
        }
        .run()
        .await
    }
}

#[derive(Debug)]
struct ServerImpl {
    rx: MessageProto<OwnedReadHalf>,
    tx: Arc<tokio::sync::Mutex<MessageProto<OwnedWriteHalf>>>,
    rpc_methods: Arc<HashMap<&'static str, Method>>,
}
impl ServerImpl {
    async fn run(mut self) -> anyhow::Result<()> {
        let mut buf = Vec::with_capacity(4096);
        loop {
            self.rx.recv(&mut buf).await?;
            let request: airup_sdk::extension::Request = ciborium::from_reader(&buf[..])?;
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
        rpc_methods: Arc<HashMap<&'static str, Method>>,
        request: airup_sdk::rpc::Request,
    ) -> airup_sdk::rpc::Response {
        match rpc_methods.get(&request.method[..]) {
            Some(method) => airup_sdk::rpc::Response::new(method(request).await),
            None => airup_sdk::rpc::Response::new::<()>(Err(airup_sdk::Error::NotImplemented)),
        }
    }
}

/// Represents to type of function pointer of an IPC method.
pub type Method = fn(airup_sdk::rpc::Request) -> MethodFuture;

/// Represents to future type of an IPC method.
pub type MethodFuture =
    Pin<Box<dyn Future<Output = Result<ciborium::Value, airup_sdk::Error>> + Send>>;
