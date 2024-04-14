//! A simple logger.
//!
//! This has some limitations and has bad performance. Being designed as an "fallback choice", the implementation aims to be
//! small.

use airup_sdk::{
    blocking::fs::DirChain,
    info::ConnectionExt,
    ipc::MessageProto,
    nonblocking::ipc::{MessageProtoRecvExt, MessageProtoSendExt},
    system::{ConnectionExt as _, LogRecord},
    Error,
};
use ciborium::cbor;
use rev_lines::RevLines;
use std::{collections::HashSet, io::Write};
use tokio::net::UnixStream;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
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
    std::fs::remove_file(&extension_socket_path).ok();
    let extension_socket = tokio::net::UnixListener::bind(&extension_socket_path)?;
    airupfx::fs::set_permission(&extension_socket_path, airupfx::fs::Permission::Socket).await?;
    let extension_socket_path_cloned = extension_socket_path.clone();

    tokio::spawn(async move {
        let methods = HashSet::from(["logger.append".into(), "logger.tail".into()]);
        match airup_rpc_conn
            .load_extension(&service_name, &extension_socket_path_cloned, methods)
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
    });

    loop {
        let connection = extension_socket.accept().await?.0;
        let connection = MessageProto::new(connection, 6 * 1024 * 1024);
        tokio::spawn(handle_connection(connection));
    }
}

async fn handle_connection(mut connection: MessageProto<UnixStream>) -> anyhow::Result<()> {
    loop {
        let message = connection.recv().await?;
        let request: airup_sdk::extension::Request = ciborium::from_reader(&message[..])?;
        if request.class == airup_sdk::extension::Request::CLASS_AIRUP_RPC {
            let req_arpc: airup_sdk::ipc::Request = request.data.deserialized()?;
            let resp_arpc = match &req_arpc.method[..] {
                "logger.append" => airup_sdk::ipc::Response::new(append(req_arpc)),
                "logger.tail" => airup_sdk::ipc::Response::new(tail(req_arpc)),
                _ => airup_sdk::ipc::Response::new::<()>(Err(airup_sdk::Error::NoSuchMethod)),
            };
            let resp = airup_sdk::extension::Response {
                id: request.id,
                data: ciborium::Value::serialized(&resp_arpc)
                    .expect("response should always serialize into a CBOR object"),
            };
            let mut buf = Vec::with_capacity(128);
            ciborium::into_writer(&resp, &mut buf)?;
            connection.send(&buf).await?;
        } else {
            let mut buf = Vec::with_capacity(64);
            let response = airup_sdk::extension::Response {
                id: request.id,
                data: cbor!({}).unwrap(),
            };
            ciborium::into_writer(&response, &mut buf)?;
            connection.send(&buf).await?;
        }
    }
}

fn dir_chain_logs() -> DirChain<'static> {
    DirChain::new(&airup_sdk::build::manifest().log_dir)
}

fn open_subject_append(subject: &str) -> std::io::Result<std::fs::File> {
    let path = dir_chain_logs().find_or_create(&format!("{subject}.fallback_logger.json"))?;

    std::fs::File::options()
        .append(true)
        .create(true)
        .open(path)
}

fn open_subject_read(subject: &str) -> std::io::Result<std::fs::File> {
    let path = dir_chain_logs()
        .find(&format!("{subject}.fallback_logger.json"))
        .ok_or_else(|| std::io::Error::from(std::io::ErrorKind::NotFound))?;

    std::fs::File::open(path)
}

fn append(req: airup_sdk::ipc::Request) -> Result<(), Error> {
    let (subject, module, msg): (String, String, Vec<u8>) = req.extract_params()?;

    let mut appender = open_subject_append(&subject).map_err(|x| Error::Io {
        message: x.to_string(),
    })?;
    let timestamp = airupfx::time::timestamp_ms();

    for m in msg.split(|x| b"\r\n\0".contains(x)) {
        let record = LogRecord {
            timestamp,
            module: module.to_owned(),
            message: String::from_utf8_lossy(m).into_owned(),
        };
        writeln!(
            appender,
            "{}",
            serde_json::to_string(&record).unwrap().as_str()
        )
        .map_err(|x| airup_sdk::Error::Io {
            message: x.to_string(),
        })?;
    }

    Ok(())
}

fn tail(req: airup_sdk::ipc::Request) -> Result<Vec<LogRecord>, Error> {
    let (subject, n): (String, usize) = req.extract_params()?;

    let reader = open_subject_read(&subject).map_err(|x| Error::Io {
        message: x.to_string(),
    })?;

    let mut result = Vec::with_capacity(n);
    for line in RevLines::new(reader).take(n) {
        result.push(
            serde_json::from_str(&line.map_err(|x| Error::Io {
                message: x.to_string(),
            })?)
            .map_err(|x| Error::Io {
                message: x.to_string(),
            })?,
        );
    }
    result.reverse();
    Ok(result)
}
