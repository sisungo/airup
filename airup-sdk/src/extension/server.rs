use super::{Request, Response};
use ciborium::cbor;
use std::{
    collections::HashMap,
    future::Future,
    io::{Read, Write},
    pin::Pin,
    sync::Arc,
};

pub struct Server {
    rpc_api: Arc<RpcApi>,
}
impl Server {
    pub fn new(rpc_api: Arc<RpcApi>) -> Self {
        Self { rpc_api }
    }

    pub async fn run(self) -> std::io::Result<()> {
        loop {
            let request: std::io::Result<Request> = tokio::task::spawn_blocking(|| {
                let mut stdin_lock = std::io::stdin().lock();
                let mut buf = [0u8; 8];
                stdin_lock.read_exact(&mut buf)?;
                let len = u64::from_le_bytes(buf);
                let mut buf = vec![0u8; len as _];
                stdin_lock.read_exact(&mut buf)?;
                let request: Request = ciborium::from_reader(&buf[..]).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid cbor")
                })?;

                Ok(request)
            })
            .await
            .unwrap();
            let request = request?;

            if request.class == Request::CLASS_AIRUP_RPC {
                let rpc_api = self.rpc_api.clone();
                tokio::spawn(async move {
                    let Request { id, class: _, data } = request;
                    let Ok(request) = data.deserialized::<crate::ipc::Request>() else {
                        return;
                    };
                    let response = match rpc_api.find(&request.method) {
                        Some(method) => crate::ipc::Response::new(method(request).await),
                        None => crate::ipc::Response::new::<()>(Err(crate::Error::NoSuchMethod)),
                    };
                    write_resp(Response {
                        id,
                        data: ciborium::Value::serialized(&response).unwrap(),
                    })
                    .ok();
                });
            } else {
                write_resp(Response {
                    id: request.id,
                    data: cbor!({}).unwrap(),
                })?;
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct RpcApi(HashMap<String, Method>);
impl RpcApi {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: String, method: Method) {
        self.0.insert(name, method);
    }

    pub fn find(&self, method: &str) -> Option<Method> {
        self.0.get(method).copied()
    }
}

pub(super) type Method = fn(crate::ipc::Request) -> MethodFuture;

/// Represents to future type of an IPC method.
pub type MethodFuture = Pin<Box<dyn Future<Output = Result<ciborium::Value, crate::Error>> + Send>>;

fn write_resp(resp: Response) -> std::io::Result<()> {
    let mut buf = Vec::with_capacity(128);
    ciborium::into_writer(&resp, &mut buf)
        .expect("caller must guarantee the response can be serialized into cbor");
    let mut lock = std::io::stdout().lock();
    lock.write_all(&buf.len().to_le_bytes())?;
    lock.write_all(&buf)?;
    lock.flush()?;
    Ok(())
}
