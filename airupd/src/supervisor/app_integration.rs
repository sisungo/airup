use super::task::TaskHandle;
use airup_sdk::{system::QueryService, Error};
use airupfx::files::Service;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait AirupdExt {
    /// Starts a service, ignoring the error if the service is already started and waits until it is active.
    async fn make_service_active(&self, name: &str) -> Result<(), Error>;

    /// Starts a service.
    async fn start_service(&self, name: &str) -> Result<Arc<dyn TaskHandle>, Error>;

    /// Queries a service.
    async fn query_service(&self, name: &str) -> Result<QueryService, Error>;

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
        let supervisor = match self.supervisors.get(name).await {
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
        match self.supervisors.get(name).await {
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

    async fn query_service(&self, name: &str) -> Result<QueryService, Error> {
        let name = name.strip_suffix(Service::SUFFIX).unwrap_or(name);
        match self.supervisors.get(name).await {
            Some(supervisor) => Ok(supervisor.query().await),
            None => Ok(QueryService::default_of(
                self.storage.services.get(name).await?,
            )),
        }
    }

    async fn stop_service(&self, name: &str) -> Result<Arc<dyn TaskHandle>, Error> {
        let name = name.strip_suffix(Service::SUFFIX).unwrap_or(name);
        match self.supervisors.get(name).await {
            Some(supervisor) => Ok(supervisor.stop().await?),
            None => {
                self.storage.services.get(name).await?;
                Err(Error::ObjectNotConfigured)
            }
        }
    }

    async fn reload_service(&self, name: &str) -> Result<Arc<dyn TaskHandle>, Error> {
        let name = name.strip_suffix(Service::SUFFIX).unwrap_or(name);
        match self.supervisors.get(name).await {
            Some(supervisor) => Ok(supervisor.reload().await?),
            None => {
                self.storage.services.get(name).await?;
                Err(Error::ObjectNotConfigured)
            }
        }
    }

    async fn interrupt_service_task(&self, name: &str) -> Result<Arc<dyn TaskHandle>, Error> {
        let name = name.strip_suffix(Service::SUFFIX).unwrap_or(name);
        match self.supervisors.get(name).await {
            Some(supervisor) => Ok(supervisor.interrupt_task().await?),
            None => {
                self.storage.services.get(name).await?;
                Err(Error::ObjectNotConfigured)
            }
        }
    }
}
