//! `system.*` APIs.

use crate::{files::Service, system::*, Error};
use std::future::Future;

pub trait ConnectionExt {
    /// Sideloads a service.
    fn sideload_service(
        &mut self,
        name: &str,
        service: &Service,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>> + Send;

    /// Starts the specified service.
    fn start_service(
        &mut self,
        name: &str,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Stops the specified service.
    fn stop_service(
        &mut self,
        name: &str,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Forces the specified service to stop.
    fn kill_service(
        &mut self,
        name: &str,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Reloads the specified service.
    fn reload_service(
        &mut self,
        name: &str,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Caches the specified service.
    fn cache_service(
        &mut self,
        name: &str,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Uncaches the specified service.
    fn uncache_service(
        &mut self,
        name: &str,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Queries the specified service.
    fn query_service(
        &mut self,
        name: &str,
    ) -> impl Future<Output = anyhow::Result<Result<QueryService, Error>>>;

    /// Interrupts current task running in specific service's supervisor.
    fn interrupt_service_task(
        &mut self,
        name: &str,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Queries information about the whole system.
    fn query_system(&mut self) -> impl Future<Output = anyhow::Result<Result<QuerySystem, Error>>>;

    /// Lists all services.
    fn list_services(&mut self)
        -> impl Future<Output = anyhow::Result<Result<Vec<String>, Error>>>;

    /// Refreshes cached system information in the `airupd` daemon.
    fn refresh(&mut self) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Deletes cached system information in the `airupd` daemon.
    fn gc(&mut self) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Powers the system off.
    fn poweroff(&mut self) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Reboots the system.
    fn reboot(&mut self) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Halts the system.
    fn halt(&mut self) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Indicates `airupd` to register the specified logger.
    fn use_logger(
        &mut self,
        name: Option<&str>,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Queries latest `n` log records from the logger.
    fn tail_logs(
        &mut self,
        subject: &str,
        n: usize,
    ) -> impl Future<Output = anyhow::Result<Result<Vec<LogRecord>, Error>>>;

    /// Enters the specific milestone.
    fn enter_milestone(
        &mut self,
        name: &str,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;

    /// Triggers the specific event.
    fn trigger_event(
        &mut self,
        event: &str,
    ) -> impl Future<Output = anyhow::Result<Result<(), Error>>>;
}
impl ConnectionExt for super::Connection {
    async fn sideload_service(
        &mut self,
        name: &str,
        service: &Service,
    ) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.sideload_service", (name, service))
            .await
    }

    async fn start_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.start_service", name).await
    }

    async fn stop_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.stop_service", name).await
    }

    async fn kill_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.kill_service", name).await
    }

    async fn cache_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.cache_service", name).await
    }

    async fn uncache_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.uncache_service", name).await
    }

    async fn reload_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.reload_service", name).await
    }

    async fn query_service(&mut self, name: &str) -> anyhow::Result<Result<QueryService, Error>> {
        self.invoke("system.query_service", name).await
    }

    async fn interrupt_service_task(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.interrupt_service_task", name).await
    }

    async fn list_services(&mut self) -> anyhow::Result<Result<Vec<String>, Error>> {
        self.invoke("system.list_services", ()).await
    }

    async fn query_system(&mut self) -> anyhow::Result<Result<QuerySystem, Error>> {
        self.invoke("system.query_system", ()).await
    }

    async fn refresh(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.refresh", ()).await
    }

    async fn gc(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.gc", ()).await
    }

    async fn poweroff(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.poweroff", ()).await
    }

    async fn reboot(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.reboot", ()).await
    }

    async fn halt(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.halt", ()).await
    }

    async fn use_logger(&mut self, name: Option<&str>) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.use_logger", name).await
    }

    async fn tail_logs(
        &mut self,
        subject: &str,
        n: usize,
    ) -> anyhow::Result<Result<Vec<LogRecord>, Error>> {
        self.invoke("system.tail_logs", (subject, n)).await
    }

    async fn enter_milestone(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.enter_milestone", name).await
    }

    async fn trigger_event(&mut self, event: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.trigger_event", event).await
    }
}
