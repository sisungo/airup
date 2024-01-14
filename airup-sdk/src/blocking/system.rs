//! `system.*` APIs.

use crate::{files::Service, system::*, Error};

pub trait ConnectionExt {
    /// Sideloads a service.
    fn sideload_service(
        &mut self,
        name: &str,
        service: &Service,
    ) -> anyhow::Result<Result<(), Error>>;

    /// Starts the specified service.
    fn start_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>>;

    /// Stops the specified service.
    fn stop_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>>;

    /// Forces the specified service to stop.
    fn kill_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>>;

    /// Reloads the specified service.
    fn reload_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>>;

    /// Caches the specified service.
    fn cache_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>>;

    /// Uncaches the specified service.
    fn uncache_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>>;

    /// Queries the specified service.
    fn query_service(&mut self, name: &str) -> anyhow::Result<Result<QueryService, Error>>;

    /// Interrupts current task running in specific service's supervisor.
    fn interrupt_service_task(&mut self, name: &str) -> anyhow::Result<Result<(), Error>>;

    /// Queries information about the whole system.
    fn query_system(&mut self) -> anyhow::Result<Result<QuerySystem, Error>>;

    /// Lists all services.
    fn list_services(&mut self) -> anyhow::Result<Result<Vec<String>, Error>>;

    /// Refreshes cached system information in the `airupd` daemon.
    fn refresh(&mut self) -> anyhow::Result<Result<(), Error>>;

    /// Deletes cached system information in the `airupd` daemon.
    fn gc(&mut self) -> anyhow::Result<Result<(), Error>>;

    /// Powers the system off.
    fn poweroff(&mut self) -> anyhow::Result<Result<(), Error>>;

    /// Reboots the system.
    fn reboot(&mut self) -> anyhow::Result<Result<(), Error>>;

    /// Halts the system.
    fn halt(&mut self) -> anyhow::Result<Result<(), Error>>;

    /// Indicates `airupd` to register the specified logger.
    fn use_logger(&mut self, name: Option<&str>) -> anyhow::Result<Result<(), Error>>;

    /// Queries latest `n` log records from the logger.
    fn tail_logs(
        &mut self,
        subject: &str,
        n: usize,
    ) -> anyhow::Result<Result<Vec<LogRecord>, Error>>;

    /// Enters the specific milestone.
    fn enter_milestone(&mut self, name: &str) -> anyhow::Result<Result<(), Error>>;

    /// Triggers the specific event.
    fn trigger_event(&mut self, event: &str) -> anyhow::Result<Result<(), Error>>;
}
impl ConnectionExt for super::Connection {
    fn sideload_service(
        &mut self,
        name: &str,
        service: &Service,
    ) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.sideload_service", (name, service))
    }

    fn start_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.start_service", name)
    }

    fn stop_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.stop_service", name)
    }

    fn kill_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.kill_service", name)
    }

    fn cache_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.cache_service", name)
    }

    fn uncache_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.uncache_service", name)
    }

    fn reload_service(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.reload_service", name)
    }

    fn query_service(&mut self, name: &str) -> anyhow::Result<Result<QueryService, Error>> {
        self.invoke("system.query_service", name)
    }

    fn interrupt_service_task(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.interrupt_service_task", name)
    }

    fn list_services(&mut self) -> anyhow::Result<Result<Vec<String>, Error>> {
        self.invoke("system.list_services", ())
    }

    fn query_system(&mut self) -> anyhow::Result<Result<QuerySystem, Error>> {
        self.invoke("system.query_system", ())
    }

    fn refresh(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.refresh", ())
    }

    fn gc(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.gc", ())
    }

    fn poweroff(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.poweroff", ())
    }

    fn reboot(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.reboot", ())
    }

    fn halt(&mut self) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.halt", ())
    }

    fn use_logger(&mut self, name: Option<&str>) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.use_logger", name)
    }

    fn tail_logs(
        &mut self,
        subject: &str,
        n: usize,
    ) -> anyhow::Result<Result<Vec<LogRecord>, Error>> {
        self.invoke("system.tail_logs", (subject, n))
    }

    fn enter_milestone(&mut self, name: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.enter_milestone", name)
    }

    fn trigger_event(&mut self, event: &str) -> anyhow::Result<Result<(), Error>> {
        self.invoke("system.trigger_event", event)
    }
}
