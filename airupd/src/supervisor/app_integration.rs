use super::task::TaskHandle;
use airupfx::{
    files::Service,
    sdk::{system::QueryResult, Error},
};
use std::sync::Arc;

#[async_trait::async_trait]
pub trait AirupdExt {
    /// Starts a service, ignoring the error if the service is already started and waits until it is active.
    async fn make_service_active(&self, name: &str) -> Result<(), Error>;

    /// Starts a service.
    async fn start_service(&self, name: &str) -> Result<Arc<dyn TaskHandle>, Error>;

    /// Queries a service.
    async fn query_service(&self, name: &str) -> Result<QueryResult, Error>;

    /// Stops a service.
    async fn stop_service(&self, name: &str) -> Result<Arc<dyn TaskHandle>, Error>;

    /// Reloads a service.
    async fn reload_service(&self, name: &str) -> Result<Arc<dyn TaskHandle>, Error>;

    /// Interrupts current running task for the specified service.
    async fn interrupt_service_task(&self, name: &str) -> Result<Arc<dyn TaskHandle>, Error>;
}
#[async_trait::async_trait]
impl AirupdExt for crate::app::Airupd {
    async fn make_service_active(&self, name: &str) -> Result<(), Error> {
        let supervisor = match self.supervisors.get(name) {
            Some(supervisor) => supervisor,
            None => {
                self.supervisors
                    .supervise(self.storage.services.get(name).await?)
                    .await?
            }
        };
        supervisor.make_active().await
    }

    async fn start_service(&self, name: &str) -> Result<Arc<dyn TaskHandle>, Error> {
        match self.supervisors.get(name) {
            Some(supervisor) => Ok(supervisor.start().await?),
            None => {
                let supervisor = self
                    .supervisors
                    .supervise(self.storage.services.get(name).await?)
                    .await?;
                Ok(supervisor.start().await?)
            }
        }
    }

    async fn query_service(&self, name: &str) -> Result<QueryResult, Error> {
        let name = name.strip_suffix(Service::SUFFIX).unwrap_or(name);
        match self.supervisors.get(name) {
            Some(supervisor) => Ok(supervisor.query().await),
            None => Ok(QueryResult::default_of(
                self.storage.services.get(name).await?,
            )),
        }
    }

    async fn stop_service(&self, name: &str) -> Result<Arc<dyn TaskHandle>, Error> {
        let name = name.strip_suffix(Service::SUFFIX).unwrap_or(name);
        match self.supervisors.get(name) {
            Some(supervisor) => Ok(supervisor.stop().await?),
            None => {
                self.storage.services.get(name).await?;
                Err(Error::ObjectNotConfigured)
            }
        }
    }

    async fn reload_service(&self, name: &str) -> Result<Arc<dyn TaskHandle>, Error> {
        let name = name.strip_suffix(Service::SUFFIX).unwrap_or(name);
        match self.supervisors.get(name) {
            Some(supervisor) => Ok(supervisor.reload().await?),
            None => {
                self.storage.services.get(name).await?;
                Err(Error::ObjectNotConfigured)
            }
        }
    }

    async fn interrupt_service_task(
        &self,
        name: &str,
    ) -> Result<Arc<dyn TaskHandle>, Error> {
        let name = name.strip_suffix(Service::SUFFIX).unwrap_or(name);
        match self.supervisors.get(name) {
            Some(supervisor) => Ok(supervisor.interrupt_task().await?),
            None => {
                self.storage.services.get(name).await?;
                Err(Error::ObjectNotConfigured)
            }
        }
    }
}
