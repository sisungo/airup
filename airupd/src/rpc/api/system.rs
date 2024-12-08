//! APIs that provides system operations.

use super::{Method, MethodFuture};
use crate::app::airupd;
use airup_sdk::{
    Error,
    files::Service,
    system::{Event, QueryService, QuerySystem},
};
use std::{collections::HashMap, hash::BuildHasher};

pub(super) fn init<H: BuildHasher>(methods: &mut HashMap<&'static str, Method, H>) {
    crate::ipc_methods!(system, [
        refresh,
        gc,
        start_service,
        query_service,
        query_system,
        stop_service,
        kill_service,
        reload_service,
        sideload_service,
        cache_service,
        uncache_service,
        interrupt_service_task,
        list_services,
        enter_milestone,
        set_instance_name,
        trigger_event,
        unregister_extension,
    ])
    .iter()
    .for_each(|(k, v)| {
        methods.insert(k, *v);
    });
}

#[airupfx::macros::api]
async fn refresh() -> Result<Vec<(String, Error)>, Error> {
    let mut errors = Vec::new();

    airupfx::env::refresh().await;
    for (name, error) in airupd().supervisors.refresh_all().await {
        errors.push((format!("service-manifest:{name}"), error));
    }

    Ok(errors)
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
    airupd().sideload_service(&name, service).await
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
async fn enter_milestone(name: String) -> Result<(), Error> {
    airupd().enter_milestone(name).await
}

#[airupfx::macros::api]
async fn set_instance_name(name: String) -> Result<(), Error> {
    airupfx::env::set_instance_name(name);
    Ok(())
}

#[airupfx::macros::api]
async fn trigger_event(event: Event) -> Result<(), Error> {
    airupd().events.trigger(event).await;
    Ok(())
}

#[airupfx::macros::api]
async fn unregister_extension(name: String) -> Result<(), Error> {
    airupd().extensions.unregister(&name)
}
