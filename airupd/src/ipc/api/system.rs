//! APIs that provides system operations.

use super::{Method, MethodFuture};
use crate::app::airupd;
use airup_sdk::{
    files::Service,
    system::{Event, LogRecord, QueryService, QuerySystem},
    Error,
};
use std::{
    collections::{HashMap, HashSet},
    hash::BuildHasher,
};

pub(super) fn init<H: BuildHasher>(methods: &mut HashMap<&'static str, Method, H>) {
    crate::ipc_methods!(
        system,
        [
            refresh,
            gc,
            start_service,
            query_service,
            query_system,
            stop_service,
            kill_service,
            reload_service,
            sideload_service,
            unsideload_service,
            cache_service,
            uncache_service,
            interrupt_service_task,
            list_services,
            append_log,
            tail_logs,
            enter_milestone,
            trigger_event,
            load_extension,
            unload_extension,
        ]
    )
    .iter()
    .for_each(|(k, v)| {
        methods.insert(k, *v);
    });
}

#[airupfx::macros::api]
async fn refresh() -> Result<(), Error> {
    airupfx::env::refresh().await;
    airupd().supervisors.refresh_all().await;

    Ok(())
}

#[airupfx::macros::api]
async fn gc() -> Result<(), Error> {
    airupd().supervisors.gc().await;
    Ok(())
}

#[airupfx::macros::api]
async fn query_service(service: String) -> Result<QueryService, Error> {
    airupd().query_service(&service).await
}

#[airupfx::macros::api]
async fn query_system() -> Result<QuerySystem, Error> {
    Ok(airupd().query_system().await)
}

#[airupfx::macros::api]
async fn start_service(service: String) -> Result<(), Error> {
    airupd().start_service(&service).await?.wait().await?;
    Ok(())
}

#[airupfx::macros::api]
async fn stop_service(service: String) -> Result<(), Error> {
    airupd().stop_service(&service).await?.wait().await?;
    Ok(())
}

#[airupfx::macros::api]
async fn kill_service(service: String) -> Result<(), Error> {
    airupd().kill_service(&service).await
}

#[airupfx::macros::api]
async fn reload_service(service: String) -> Result<(), Error> {
    airupd().reload_service(&service).await?.wait().await?;
    Ok(())
}

#[airupfx::macros::api]
async fn interrupt_service_task(service: String) -> Result<(), Error> {
    airupd()
        .interrupt_service_task(&service)
        .await?
        .wait()
        .await
        .map(|_| ())
}

#[airupfx::macros::api]
async fn sideload_service(name: String, service: Service, ovrd: bool) -> Result<(), Error> {
    airupd().storage.services.load(&name, service, ovrd)
}

#[airupfx::macros::api]
async fn unsideload_service(name: String) -> Result<(), Error> {
    airupd().storage.services.unload(&name)
}

#[airupfx::macros::api]
async fn cache_service(service: String) -> Result<(), Error> {
    airupd().cache_service(&service).await
}

#[airupfx::macros::api]
async fn uncache_service(service: String) -> Result<(), Error> {
    airupd().uncache_service(&service).await
}

#[airupfx::macros::api]
async fn list_services() -> Result<Vec<String>, Error> {
    Ok(airupd().storage.services.list().await)
}

#[airupfx::macros::api]
async fn tail_logs(subject: String, n: usize) -> Result<Vec<LogRecord>, Error> {
    if n > 1024 + 512 {
        return Err(Error::invalid_params(
            "method `system.tail_logs(subject, n)` only accepts `n < 1024 + 512`",
        ));
    }

    let queried = crate::logger::tail(&subject, n)
        .await
        .map_err(airup_sdk::Error::custom)?;

    Ok(queried)
}

#[airupfx::macros::api]
async fn append_log(subject: String, module: String, message: String) -> Result<(), Error> {
    crate::logger::write(&subject, &module, message.as_bytes())
        .await
        .map_err(airup_sdk::Error::custom)?;

    Ok(())
}

#[airupfx::macros::api]
async fn enter_milestone(name: String) -> Result<(), Error> {
    airupd().enter_milestone(name).await
}

#[airupfx::macros::api]
async fn trigger_event(event: Event) -> Result<(), Error> {
    airupd().events.trigger(event).await;
    Ok(())
}

#[airupfx::macros::api]
async fn load_extension(name: String, path: String, methods: HashSet<String>) -> Result<(), Error> {
    airupd().extensions.load(name, &path, methods).await
}

#[airupfx::macros::api]
async fn unload_extension(name: String) -> Result<(), Error> {
    airupd().extensions.unload(&name).await
}
