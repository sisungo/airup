//! APIs that provides system operations.

use super::{Method, MethodFuture};
use crate::{app::airupd, ipc::SessionContext};
use airup_sdk::{
    files::Service,
    ipc::Request,
    system::{LogRecord, QueryService, QuerySystem},
    Error,
};
use std::{collections::HashMap, hash::BuildHasher, sync::Arc};

pub fn init<H: BuildHasher>(methods: &mut HashMap<&'static str, Method, H>) {
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
            use_logger,
            tail_logs,
            enter_milestone,
            trigger_event,
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
async fn sideload_service(name: String, service: Service) -> Result<(), Error> {
    airupd().storage.services.load(&name, service)
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
async fn use_logger(logger: Option<String>) -> Result<(), Error> {
    match logger {
        Some(name) => {
            airupd().logger.set_logger_by_name(&name).await?;
        }
        None => {
            airupd().logger.remove_logger().await;
        }
    }
    Ok(())
}

#[airupfx::macros::api]
async fn tail_logs(subject: String, n: usize) -> Result<Vec<LogRecord>, Error> {
    let queried = airupd()
        .logger
        .tail(&subject, n)
        .await
        .map_err(airup_sdk::Error::custom)?;

    Ok(queried)
}

#[airupfx::macros::api]
async fn enter_milestone(name: String) -> Result<(), Error> {
    airupd().enter_milestone(name).await
}

#[airupfx::macros::api]
async fn trigger_event(event: String) -> Result<(), Error> {
    airupd().events.trigger(event).await;
    Ok(())
}
