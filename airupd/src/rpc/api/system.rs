//! APIs that provides system operations.

use super::MethodFuture;
use crate::{app::airupd, rpc::route::Router};
use airup_sdk::{
    Error,
    files::Service,
    system::{Event, QueryService, QuerySystem},
};

pub fn router() -> Router {
    Router::new()
        .route("refresh", refresh)
        .route("gc", gc)
        .route("start_service", start_service)
        .route("query_service", query_service)
        .route("query_system", query_system)
        .route("stop_service", stop_service)
        .route("kill_service", kill_service)
        .route("reload_service", reload_service)
        .route("sideload_service", sideload_service)
        .route("cache_service", cache_service)
        .route("uncache_service", uncache_service)
        .route("interrupt_service_task", interrupt_service_task)
        .route("list_services", list_services)
        .route("enter_milestone", enter_milestone)
        .route("set_instance_name", set_instance_name)
        .route("trigger_event", trigger_event)
        .route("unregister_extension", unregister_extension)
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
